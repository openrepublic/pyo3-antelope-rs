use antelope::{
    chain::{
        abi::{ABIResolveError, ABIResolvedType, ABITypeResolver, ABIView, TypeModifier},
        asset::{Asset, ExtendedAsset, Symbol, SymbolCode},
        checksum::{Checksum160, Checksum256, Checksum512},
        name::Name,
        public_key::PublicKey,
        signature::Signature,
        time::{BlockTimestamp, TimePoint, TimePointSec},
        varint::VarUint32,
    },
    serializer::{packer::Float128, Decoder},
};
use pyo3::{
    exceptions::{PyNotImplementedError, PyValueError},
    prelude::*,
    types::{PyBytes, PyDict, PyList},
    IntoPyObjectExt,
};
use thiserror::Error;

use crate::proxies::{
    asset::{Asset as PyAsset, ExtendedAsset as PyExtendedAsset},
    checksums::{Checksum160 as PyChecksum160, Checksum256 as PyChecksum256, Checksum512 as PyChecksum512},
    name::Name as PyName,
    public_key::PublicKey as PyPublicKey,
    signature::Signature as PySignature,
    sym::Symbol as PySymbol,
    sym_code::SymbolCode as PySymbolCode,
};

#[derive(Clone, Debug, Default)]
struct DecodePath(Vec<String>);

impl DecodePath {
    fn push<S: Into<String>>(&mut self, seg: S) {
        self.0.push(seg.into())
    }
    fn pop(&mut self) {
        self.0.pop();
    }
    fn as_str(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("type-resolution error at `{path}`: {source}")]
    Resolve {
        path: String,
        #[source]
        source: ABIResolveError,
    },

    #[error("malformed input while unpacking `{what}` at `{path}`: {err}")]
    Unpack {
        what: String,
        path: String,
        err: String,
    },

    #[error("unknown std type `{name}` at `{path}`")]
    UnknownStdType { name: String, path: String },

    #[error("unknown type `{name}` at `{path}`")]
    UnknownType { name: String, path: String },
}

impl From<DecodeError> for PyErr {
    fn from(value: DecodeError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

pub fn decode_abi_type<'py, ABI>(
    py: Python<'py>,
    abi: &ABI,
    type_name: &str,
    decoder: &mut Decoder<'_>,
) -> PyResult<Bound<'py, PyAny>>
where
    ABI: ABIView + ABITypeResolver,
{
    let mut meta = abi
        .resolve_type(type_name)
        .map_err(|e| DecodeError::Resolve {
            path: type_name.to_string(),
            source: e,
        })?;

    let mut path = DecodePath::default();
    path.push(type_name);

    decode_with_meta(py, abi, &mut meta, decoder, &mut path)
}

fn decode_with_meta<'py, ABI>(
    py: Python<'py>,
    abi: &ABI,
    meta: &mut ABIResolvedType,
    decoder: &mut Decoder<'_>,
    path: &mut DecodePath,
) -> PyResult<Bound<'py, PyAny>>
where
    ABI: ABIView + ABITypeResolver,
{
    if !meta.modifiers.is_empty() {
        let this_mod = meta.modifiers.remove(0);
        match this_mod {
            TypeModifier::Optional => {
                let mut flag: u8 = 0;
                decoder
                    .unpack(&mut flag)
                    .map_err(|e| DecodeError::Unpack {
                        what: "optional-flag".into(),
                        path: path.as_str(),
                        err: e.to_string(),
                    })?;

                if flag == 0 {
                    return Ok(py.None().into_bound(py));
                }
                path.push("some");
                let res = decode_with_meta(py, abi, meta, decoder, path);
                path.pop();
                return res;
            }
            TypeModifier::Array => {
                let mut len_vu: VarUint32 = VarUint32::default();
                decoder
                    .unpack(&mut len_vu)
                    .map_err(|e| DecodeError::Unpack {
                        what: "array-length".into(),
                        path: path.as_str(),
                        err: e.to_string(),
                    })?;
                let len = len_vu.value() as usize;

                let list = PyList::empty(py);
                for i in 0..len {
                    path.push(format!("[{i}]"));
                    let item = decode_with_meta(py, abi, meta, decoder, path)?;
                    list.append(item)?;
                    path.pop();
                }
                return Ok(list.into_any());
            }
            TypeModifier::Extension => {
                if decoder.remaining() == 0 {
                    return Ok(py.None().into_bound(py));
                }
                path.push("extension");
                let res = decode_with_meta(py, abi, meta, decoder, path);
                path.pop();
                return res;
            }
        }
    }

    if meta.is_std {
        return decode_std(py, meta, decoder, path);
    }

    if let Some(var_meta) = &meta.is_variant {
        let mut idx_vu: VarUint32 = VarUint32::default();
        decoder
            .unpack(&mut idx_vu)
            .map_err(|e| DecodeError::Unpack {
                what: "variant-index".into(),
                path: path.as_str(),
                err: e.to_string(),
            })?;
        let idx = idx_vu.value() as usize;

        let inner_type_name = var_meta.types.get(idx).ok_or_else(|| {
            DecodeError::UnknownType {
                name: format!("variant-idx {idx}"),
                path: path.as_str(),
            }
        })?;

        path.push(format!("variant({idx})"));
        let payload = decode_abi_type(py, abi, inner_type_name, decoder)?;
        path.pop();

        if let Ok(dict) = payload.downcast::<PyDict>() {
            dict.set_item("type", inner_type_name)?;
            return dict.into_bound_py_any(py);
        }
        return Ok(payload);
    }

    if let Some(struct_def) = &meta.is_struct {
        let dict = if !struct_def.base.is_empty() {
            let val = decode_abi_type(py, abi, &struct_def.base, decoder)?;
            val.downcast()?.to_owned()
        } else {
            PyDict::new(py)
        };
        for field in &struct_def.fields {
            path.push(field.name.clone());
            let val = decode_abi_type(py, abi, &field.r#type, decoder)?;
            dict.set_item(&field.name, val)?;
            path.pop();
        }
        return dict.into_bound_py_any(py);
    }

    Err(DecodeError::UnknownType {
        name: meta.resolved_name.clone(),
        path: path.as_str(),
    }
    .into())
}

fn decode_std<'py>(
    py: Python<'py>,
    meta: &ABIResolvedType,
    decoder: &mut Decoder<'_>,
    path: &DecodePath,
) -> PyResult<Bound<'py, PyAny>> {
    macro_rules! unpack_prim {
        ($t:ty) => {{
            let mut tmp: $t = Default::default();
            decoder
                .unpack(&mut tmp)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            tmp.into_bound_py_any(py)
        }};
    }

    match meta.resolved_name.as_str() {
        "bool" => unpack_prim!(bool),
        "uint8" => unpack_prim!(u8),
        "uint16" => unpack_prim!(u16),
        "uint32" => unpack_prim!(u32),
        "uint64" => unpack_prim!(u64),
        "uint128" => unpack_prim!(u128),
        "int8" => unpack_prim!(i8),
        "int16" => unpack_prim!(i16),
        "int32" => unpack_prim!(i32),
        "int64" => unpack_prim!(i64),
        "int128" => unpack_prim!(i128),
        "varuint32" => {
            let mut vu: VarUint32 = VarUint32::default();
            decoder
                .unpack(&mut vu)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            vu.value().into_bound_py_any(py)
        }
        "varint32" => Err(PyNotImplementedError::new_err("varint32 decoding not implemented")),
        "float32" => unpack_prim!(f32),
        "float64" => unpack_prim!(f64),
        "float128" => {
            let mut f: Float128 = Default::default();
            decoder
                .unpack(&mut f)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyBytes::new(py, &f.data).into_bound_py_any(py)
        }
        "time_point" => {
            let mut tp: TimePoint = Default::default();
            decoder
                .unpack(&mut tp)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            tp.elapsed.into_bound_py_any(py)
        }
        "time_point_sec" => {
            let mut tp: TimePointSec = Default::default();
            decoder
                .unpack(&mut tp)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            tp.seconds.into_bound_py_any(py)
        }
        "block_timestamp_type" => {
            let mut bt: BlockTimestamp = Default::default();
            decoder
                .unpack(&mut bt)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            bt.slot.into_bound_py_any(py)
        }
        "name" => {
            let mut n: Name = Default::default();
            decoder
                .unpack(&mut n)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyName { inner: n }.into_bound_py_any(py)
        }
        "bytes" => {
            let mut v: Vec<u8> = Vec::new();
            decoder
                .unpack(&mut v)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyBytes::new(py, v.as_slice()).into_bound_py_any(py)
        }
        "string" => {
            let mut s: String = String::new();
            decoder
                .unpack(&mut s)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            s.into_bound_py_any(py)
        }
        "checksum160" => {
            let mut sum: Checksum160 = Default::default();
            decoder
                .unpack(&mut sum)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyChecksum160 { inner: sum }.into_bound_py_any(py)
        }
        "checksum256" => {
            let mut sum: Checksum256 = Default::default();
            decoder
                .unpack(&mut sum)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyChecksum256 { inner: sum }.into_bound_py_any(py)
        }
        "checksum512" => {
            let mut sum: Checksum512 = Default::default();
            decoder
                .unpack(&mut sum)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyChecksum512 { inner: sum }.into_bound_py_any(py)
        }
        "public_key" => {
            let mut pk: PublicKey = Default::default();
            decoder
                .unpack(&mut pk)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyPublicKey { inner: pk }.into_bound_py_any(py)
        }
        "signature" => {
            let mut sig: Signature = Default::default();
            decoder
                .unpack(&mut sig)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PySignature { inner: sig }.into_bound_py_any(py)
        }
        "symbol" => {
            let mut sym: Symbol = Default::default();
            decoder
                .unpack(&mut sym)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PySymbol { inner: sym }.into_bound_py_any(py)
        }
        "symbol_code" => {
            let mut sc: SymbolCode = Default::default();
            decoder
                .unpack(&mut sc)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PySymbolCode { inner: sc }.into_bound_py_any(py)
        }
        "asset" => {
            let mut asset: Asset = Default::default();
            decoder
                .unpack(&mut asset)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;
            PyAsset { inner: asset }.into_bound_py_any(py)
        }
        "extended_asset" => {
            let mut ext: ExtendedAsset = Default::default();
            decoder
                .unpack(&mut ext)
                .map_err(|e| DecodeError::Unpack {
                    what: meta.resolved_name.clone(),
                    path: path.as_str(),
                    err: e.to_string(),
                })?;

            PyExtendedAsset::from(ext)
                .into_bound_py_any(py)
        }
        _ => Err(DecodeError::UnknownStdType {
            name: meta.resolved_name.clone(),
            path: path.as_str(),
        }
        .into()),
    }
}
