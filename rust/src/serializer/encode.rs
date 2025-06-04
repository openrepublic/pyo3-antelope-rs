use std::str::FromStr;

use antelope::{
    chain::{
        abi::{
            ABIResolveError, ABIResolvedType, ABITypeResolver, ABIView, AbiVariant, TypeModifier,
        },
        asset::{Asset, ExtendedAsset, Symbol, SymbolCode},
        checksum::{Checksum160, Checksum256, Checksum512},
        key_type::KeyType,
        name::Name,
        public_key::PublicKey,
        signature::Signature,
        time::{BlockTimestamp, TimePoint, TimePointSec},
        varint::VarUint32,
    },
    serializer::{packer::Float128, Encoder, Packer},
};
use pyo3::{
    exceptions::{PyNotImplementedError, PyTypeError, PyValueError},
    types::{PyAnyMethods, PyDict, PyList, PyListMethods},
    Bound, PyAny, PyErr, PyResult,
};
use thiserror::Error;

use crate::proxies::{
    asset::{PyAsset, PyExtendedAsset},
    checksums::{
        PyChecksum160, PyChecksum256, PyChecksum512,
    },
    name::PyName,
    public_key::PyPublicKey,
    signature::PySignature,
    sym::PySymbol,
    sym_code::PySymbolCode,
};

#[derive(Clone, Debug, Default)]
struct EncodePath(Vec<String>);

impl EncodePath {
    fn push<S: Into<String>>(&mut self, s: S) {
        self.0.push(s.into());
    }
    fn pop(&mut self) {
        self.0.pop();
    }
    fn as_str(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("type-resolution error at `{path}`: {source}")]
    Resolve {
        path: String,
        #[source]
        source: ABIResolveError,
    },

    #[error("parse error for `{type_name}` at `{path}` (value `{value}`): {err}")]
    Parse {
        type_name: String,
        value: String,
        path: String,
        err: String,
    },

    #[error("unpack error for `{type_name}` at `{path}`: {err}")]
    UnpackError {
        type_name: String,
        path: String,
        err: String,
    },

    #[error("python type mismatch at `{path}`; expected {expected}")]
    TypeMismatch { path: String, expected: String },

    #[error("unknown std type `{name}` at `{path}`")]
    UnknownStdType { name: String, path: String },

    #[error("extract error for `{type_name}` at `{path}` (value `{value}`)")]
    ExtractError {
        type_name: String,
        path: String,
        value: String,
    },

    #[error("malformed dict for `{type_name}` at `{path}`: {expected}")]
    MalformedDict {
        type_name: String,
        path: String,
        expected: String,
    },

    #[error("unknown type `{name}` at `{path}`")]
    UnknownType { name: String, path: String },
}

impl From<EncodeError> for PyErr {
    fn from(e: EncodeError) -> Self {
        PyValueError::new_err(e.to_string())
    }
}

pub fn encode_abi_type<'py, ABI>(
    abi: &ABI,
    type_name: &str,
    value: &Bound<'py, PyAny>,
    encoder: &mut Encoder,
) -> PyResult<usize>
where
    ABI: ABIView + ABITypeResolver,
{
    let mut meta = abi
        .resolve_type(type_name)
        .map_err(|e| EncodeError::Resolve {
            path: type_name.to_string(),
            source: e,
        })?;

    let mut path = EncodePath::default();
    path.push(type_name);

    encode_with_meta(abi, &mut meta, value, encoder, &mut path)
}

fn encode_with_meta<'py, ABI>(
    abi: &ABI,
    meta: &mut ABIResolvedType,
    value: &Bound<'py, PyAny>,
    encoder: &mut Encoder,
    path: &mut EncodePath,
) -> PyResult<usize>
where
    ABI: ABIView + ABITypeResolver,
{
    if !meta.modifiers.is_empty() {
        let this_mod = meta.modifiers.remove(0);

        match this_mod {
            TypeModifier::Optional => {
                if value.is_none() {
                    return Ok((0u8).pack(encoder));
                }
                let mut size = (1u8).pack(encoder);
                path.push("some");
                size += encode_with_meta(abi, meta, value, encoder, path)?;
                path.pop();
                return Ok(size);
            }

            TypeModifier::Array => {
                let seq = value
                    .downcast::<PyList>()
                    .map_err(|_| EncodeError::TypeMismatch {
                        path: path.as_str(),
                        expected: "list/tuple".into(),
                    })?;

                let len = seq.len() as u32;
                let mut size = VarUint32::new(len).pack(encoder);

                for (i, item) in seq.iter().enumerate() {
                    path.push(format!("[{i}]"));
                    size += encode_with_meta(abi, meta, &item, encoder, path)?;
                    path.pop();
                }
                return Ok(size);
            }

            TypeModifier::Extension => {
                if value.is_none() {
                    return Ok(0);
                }
                path.push("extension");
                let sz = encode_with_meta(abi, meta, value, encoder, path)?;
                path.pop();
                return Ok(sz);
            }
        }
    }

    if meta.is_std {
        return encode_std(meta, value, encoder, path);
    }

    if let Some(var_meta) = &meta.is_variant {
        let (idx, sel_ty) = detect_variant(path, abi, var_meta, value)?;

        let mut size = VarUint32::new(idx as u32).pack(encoder);

        path.push(format!("variant({idx})"));
        size += encode_abi_type(abi, &sel_ty, value, encoder)?;
        path.pop();

        return Ok(size);
    }

    if let Some(struct_def) = &meta.is_struct {
        let dict = value
            .downcast::<PyDict>()
            .map_err(|_| EncodeError::TypeMismatch {
                path: path.as_str(),
                expected: "dict".into(),
            })?;

        let mut size = 0usize;

        /* base first */
        if !struct_def.base.is_empty() {
            size += encode_abi_type(abi, &struct_def.base, value, encoder)?;
        }

        /* fields */
        for field in &struct_def.fields {
            let ty = &field.r#type;
            let maybe_val = dict.get_item(&field.name);

            let field_val = if let Ok(v) = maybe_val {
                v
            } else if ty.ends_with('$') {
                continue; // extension field absent is fine
            } else {
                return Err(EncodeError::MalformedDict {
                    type_name: meta.resolved_name.clone(),
                    path: path.as_str(),
                    expected: format!("missing `{}`", field.name),
                }
                .into());
            };

            path.push(field.name.clone());
            size += encode_abi_type(abi, ty, &field_val, encoder)?;
            path.pop();
        }
        return Ok(size);
    }

    Err(EncodeError::UnknownType {
        name: meta.resolved_name.clone(),
        path: path.as_str(),
    }
    .into())
}

fn encode_std<'py>(
    meta: &ABIResolvedType,
    value: &Bound<'py, PyAny>,
    encoder: &mut Encoder,
    path: &EncodePath,
) -> PyResult<usize> {
    macro_rules! extract {
        ($t:ty) => {
            value
                .extract::<$t>()
                .map_err(|_| EncodeError::ExtractError {
                    type_name: meta.resolved_name.clone(),
                    value: value.to_string(),
                    path: path.as_str(),
                })
        };
    }

    macro_rules! simple {
        ($t:ty) => {{
            let v: $t = extract!($t)?;
            Ok(v.pack(encoder))
        }};
    }

    match meta.resolved_name.as_str() {
        "bool" => {
            let flag = value.is_truthy()?;
            Ok(flag.pack(encoder))
        }
        "uint8" => simple!(u8),
        "uint16" => simple!(u16),
        "uint32" => simple!(u32),
        "uint64" => simple!(u64),
        "uint128" => simple!(u128),
        "int8" => simple!(i8),
        "int16" => simple!(i16),
        "int32" => simple!(i32),
        "int64" => simple!(i64),
        "int128" => simple!(i128),
        "varuint32" => {
            let v: u32 = extract!(u32)?;
            Ok(VarUint32::new(v).pack(encoder))
        }
        "varint32" => Err(PyNotImplementedError::new_err("varint32")),
        "float32" => simple!(f32),
        "float64" => simple!(f64),
        "float128" => {
            let v: [u8; 16] = extract!([u8; 16])?;
            Ok(Float128::new(v).pack(encoder))
        }
        "time_point" => {
            let tp = if let Ok(elapsed) = extract!(u64) {
                Ok(TimePoint { elapsed })
            } else if let Ok(s) = extract!(String) {
                TimePoint::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "uint64/string".into(),
                })
            }?;
            Ok(tp.pack(encoder))
        }
        "time_point_sec" => {
            let tp = if let Ok(sec) = extract!(u32) {
                Ok(TimePointSec { seconds: sec })
            } else if let Ok(s) = extract!(String) {
                TimePointSec::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "uint32/string".into(),
                })
            }?;
            Ok(tp.pack(encoder))
        }
        "block_timestamp_type" => {
            let bt = if let Ok(slot) = extract!(u32) {
                Ok(BlockTimestamp { slot })
            } else if let Ok(s) = extract!(String) {
                BlockTimestamp::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "uint32/string".into(),
                })
            }?;
            Ok(bt.pack(encoder))
        }
        "name" => {
            let n = if let Ok(v) = extract!(u64) {
                Ok(Name::from(v))
            } else if let Ok(s) = extract!(String) {
                Name::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(pyname) = value.extract::<PyName>() {
                Ok(pyname.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "uint64/string".into(),
                })
            }?;
            Ok(n.pack(encoder))
        }
        "bytes" => {
            let v: Vec<u8> = extract!(Vec<u8>)?;
            Ok(v.pack(encoder))
        }
        "string" => {
            let s = if let Ok(v) = extract!(String) {
                Ok(v)
            } else if let Ok(enc) = extract!(Vec<u8>) {
                String::from_utf8(enc.clone()).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: String::from_utf8_lossy(&enc).to_string(),
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "string/bytes".into(),
                })
            }?;
            Ok(s.pack(encoder))
        }
        "checksum160" => {
            let sum = if let Ok(raw) = value.extract::<[u8; 20]>() {
                Ok(Checksum160::from(raw))
            } else if let Ok(hex) = extract!(String) {
                Checksum160::from_str(&hex).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: hex,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PyChecksum160>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "bytes[20]/string".into(),
                })
            }?;
            Ok(sum.pack(encoder))
        }
        "checksum256" => {
            let sum = if let Ok(raw) = value.extract::<[u8; 32]>() {
                Ok(Checksum256::from(raw))
            } else if let Ok(hex) = extract!(String) {
                Checksum256::from_str(&hex).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: hex,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PyChecksum256>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "bytes[32]/string".into(),
                })
            }?;
            Ok(sum.pack(encoder))
        }
        "checksum512" => {
            let sum = if let Ok(raw) = value.extract::<[u8; 64]>() {
                Ok(Checksum512::from(raw))
            } else if let Ok(hex) = extract!(String) {
                Checksum512::from_str(&hex).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: hex,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PyChecksum512>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "bytes[64]/string".into(),
                })
            }?;
            Ok(sum.pack(encoder))
        }
        "public_key" => {
            let key = if let Ok((raw, kt)) = value.extract::<(Vec<u8>, u8)>() {
                let ktype =
                    KeyType::try_from(kt).map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(PublicKey::from((raw, ktype)))
            } else if let Ok(raw) = value.extract::<Vec<u8>>() {
                let mut k = PublicKey::default();
                k.unpack(&raw).map_err(|e| EncodeError::UnpackError {
                    type_name: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
                Ok(k)
            } else if let Ok(hex) = extract!(String) {
                PublicKey::from_str(&hex).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: hex,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PyPublicKey>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "tuple(bytes,int)/bytes/string".into(),
                })
            }?;
            Ok(key.pack(encoder))
        }
        "signature" => {
            let sig = if let Ok(raw) = value.extract::<Vec<u8>>() {
                let mut s = Signature::default();
                s.unpack(&raw).map_err(|e| EncodeError::UnpackError {
                    type_name: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
                Ok(s)
            } else if let Ok(hex) = extract!(String) {
                Signature::from_str(&hex).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: hex,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PySignature>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "bytes/string".into(),
                })
            }?;
            Ok(sig.pack(encoder))
        }
        "symbol" => {
            let sym = if let Ok(v) = extract!(u64) {
                Ok(Symbol::from(v))
            } else if let Ok(s) = extract!(String) {
                Symbol::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PySymbol>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "uint64/string".into(),
                })
            }?;
            Ok(sym.pack(encoder))
        }
        "symbol_code" => {
            let sc = if let Ok(v) = extract!(u64) {
                SymbolCode::try_from(v).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: v.to_string(),
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(s) = extract!(String) {
                SymbolCode::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PySymbolCode>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "uint64/string".into(),
                })
            }?;
            Ok(sc.pack(encoder))
        }
        "asset" => {
            let asset = if let Ok((amount, symbol)) = value.extract::<(i64, u64)>() {
                let sym = Symbol::from(symbol);
                Asset::try_from((amount, sym)).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: format!("{amount},{symbol}"),
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(s) = extract!(String) {
                Asset::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(d) = value.downcast::<PyDict>() {
                let amount = d.get_item("amount")?;
                let symbol = d.get_item("symbol")?;
                if let (Ok(a), Ok(s)) = (amount.extract::<i64>(), symbol.extract::<u64>()) {
                    let sym = Symbol::from(s);
                    Asset::try_from((a, sym)).map_err(|e| EncodeError::Parse {
                        type_name: meta.resolved_name.clone(),
                        value: format!("{a},{s}"),
                        path: path.as_str(),
                        err: e.to_string(),
                    })
                } else {
                    Err(EncodeError::MalformedDict {
                        type_name: meta.resolved_name.clone(),
                        path: path.as_str(),
                        expected: "amount(int64) & symbol(uint64)".into(),
                    })
                }
            } else if let Ok(py) = value.extract::<PyAsset>() {
                Ok(py.inner)
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "tuple/string/dict".into(),
                })
            }?;
            Ok(asset.pack(encoder))
        }
        "extended_asset" => {
            let ext = if let Ok(s) = extract!(String) {
                ExtendedAsset::from_str(&s).map_err(|e| EncodeError::Parse {
                    type_name: meta.resolved_name.clone(),
                    value: s,
                    path: path.as_str(),
                    err: e.to_string(),
                })
            } else if let Ok(py) = value.extract::<PyExtendedAsset>() {
                Ok(py.into())
            } else {
                Err(EncodeError::TypeMismatch {
                    path: path.as_str(),
                    expected: "string/ExtendedAsset".into(),
                })
            }?;
            Ok(ext.pack(encoder))
        }
        _ => Err(EncodeError::UnknownStdType {
            name: meta.resolved_name.clone(),
            path: path.as_str(),
        }
        .into()),
    }
}

fn detect_variant<'py>(
    path: &EncodePath,
    abi: &impl ABITypeResolver,
    var_meta: &AbiVariant,
    value: &Bound<'py, PyAny>,
) -> PyResult<(usize, String)> {
    // struct variant
    if let Ok(d) = value.downcast::<PyDict>() {
        let tag_py = d.get_item("type")?;
        let tag: String = tag_py.extract().map_err(|_| EncodeError::TypeMismatch {
            path: format!("{}['type']", path.as_str()),
            expected: "string".into(),
        })?;

        let idx = var_meta
            .types
            .iter()
            .position(|t| **t == tag)
            .ok_or_else(|| EncodeError::UnknownType {
                name: var_meta.name.clone(),
                path: path.as_str(),
            })?;

        // everything except "type" is considered the payload; allow both
        //   {type:'packed_transaction', ...fields }
        //   {type:'checksum256', value:'abcd…'}

        return Ok((idx, var_meta.types[idx].clone()));
    }

    // std type variant
    let mut candidate = None;
    for (idx, ty) in var_meta.types.iter().enumerate() {
        let meta = abi.resolve_type(ty).map_err(|e| EncodeError::Resolve {
            path: ty.to_string(),
            source: e,
        })?;
        if !meta.is_std {
            continue;
        } // we only auto-detect std types

        if candidate.is_some() {
            return Err(PyValueError::new_err(format!(
                "Ambiguous ABI variant detected {}",
                var_meta.name
            )));
        }
        candidate = Some((idx, ty.clone()));
    }

    if let Some((idx, ty)) = candidate {
        return Ok((idx, ty));
    }

    Err(PyTypeError::new_err(format!(
        "Could not detect variant type: {}",
        var_meta.name
    )))
}
