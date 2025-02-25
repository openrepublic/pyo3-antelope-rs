use antelope::chain::abi::{ABIResolvedType, AbiStruct, ABI};
use antelope::chain::{Encoder, Packer};
use antelope::chain::asset::{Asset, ExtendedAsset, Symbol, SymbolCode};
use antelope::chain::checksum::{Checksum160, Checksum256, Checksum512};
use antelope::chain::name::Name;
use antelope::chain::public_key::PublicKey;
use antelope::chain::signature::Signature;
use antelope::chain::varint::VarUint32;
use pyo3::{PyResult, Python};
use pyo3::exceptions::{PyKeyError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use crate::abi_store::ABIS;
use crate::types::ActionDataTypes;
use crate::utils::str_to_timestamp;

pub fn encode_abi_type(
    py: Python,
    abi: &ABI,
    field_type: &String,
    field_value: &ActionDataTypes,
    encoder: &mut Encoder
) -> PyResult<usize> {
    let mut size: usize = 0;
    let original_field_type = field_type.clone();
    let mut field_type = field_type.clone();

    // Handle optional
    if field_type.ends_with('?') {
        // Remove optional type hint
        field_type.pop();
        match field_value {
            ActionDataTypes::None => {
                size += 0u8.pack(encoder);
                return Ok(size);
            },
            _ => {
                size += 1u8.pack(encoder);
            }
        }
    }

    // Handle array
    if field_type.ends_with("[]") {
        // Remove list type hint
        field_type.truncate(field_type.len().saturating_sub(2));
        return match field_value {
            ActionDataTypes::List(py_list) => {
                let l: Vec<ActionDataTypes> = py_list.extract(py)?;
                size += VarUint32::new(l.len() as u32).pack(encoder);
                for value in l {
                    size += encode_abi_type(py, abi, &field_type, &value, encoder)?;
                }
                Ok(size)
            }
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "Expected list value for field {}: {:?}",
                original_field_type, field_value
            ))),
        };
    }

    size += match field_value {
        ActionDataTypes::Bool(val) => {
            Ok(val.pack(encoder))
        }

        ActionDataTypes::Int(val) => {
            match field_type.as_str() {
                "int8" => {
                    let v: i8 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "int16" => {
                    let v: i16 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "int32" => {
                    let v: i32 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "int64" => {
                    let v: i64 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "int128" => {
                    let v: i128 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "uint8" => {
                    let v: u8 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "uint16" => {
                    let v: u16 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "uint32" => {
                    let v: u32 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "uint64" => {
                    let v: u64 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "uint128" => {
                    let v: u128 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "varuint32" => {
                    let v: u32 = val.extract(py)?;
                    let v = VarUint32::new(v);
                    Ok(v.pack(encoder))
                }
                "time_point_sec" => {
                    let v: u32 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "name" | "account_name" => {
                    let v: u64 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                other => Err(PyErr::new::<PyValueError, _>(format!(
                    "Unexpected integer type for field '{}'",
                    other
                ))),
            }
        }

        ActionDataTypes::Float(val) => {
            match field_type.as_str() {
                "float32" => {
                    let v: f32 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                "float64" => {
                    let v: f64 = val.extract(py)?;
                    Ok(v.pack(encoder))
                }
                other => Err(PyErr::new::<PyValueError, _>(format!(
                    "Unexpected float type for field '{}'",
                    other
                ))),
            }
        }

        ActionDataTypes::Bytes(val) => {
            Ok(val.pack(encoder))
        }

        ActionDataTypes::String(val) => {
            match field_type.as_str() {
                "string" => {
                    Ok(val.pack(encoder))
                }
                "time_point_sec" => {
                    let ts = str_to_timestamp(val.as_str());
                    Ok(ts.pack(encoder))
                }
                "name" | "account_name" => {
                    let name = Name::from_string(val)
                        .map_err(|err| PyErr::new::<PyValueError, _>(format!("Could not parse Name \"{}\": {}", val, err)))?;

                    Ok(name.pack(encoder))
                }
                "symbol_code" => {
                    let scode = SymbolCode::from_string(val)
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Could not parse SymbolCode \"{}\": {}",
                            val, e
                        )))?;
                    Ok(scode.pack(encoder))
                }
                "symbol" => {
                    let sym = Symbol::from_string(val)
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Could not parse Symbol \"{}\": {}",
                            val, e
                        )))?;
                    Ok(sym.pack(encoder))
                }
                "asset" => {
                    let asset = Asset::from_string(val)
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Could not parse Asset \"{}\": {}",
                            val, e
                        )))?;
                    Ok(asset.pack(encoder))
                }
                "extended_asset" => {
                    let ex_asset = ExtendedAsset::from_string(val)
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Could not parse ExtendedAsset \"{}\": {}", val, e
                        )))?;
                    Ok(ex_asset.pack(encoder))
                }
                "checksum160" | "rd160" => {
                    let c = Checksum160::from_hex(val.as_str())
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Wrong encoding for checksum160 string: {}",
                            e
                        )))?;
                    Ok(c.pack(encoder))
                }
                "checksum256" | "sha256" => {
                    let c = Checksum256::from_hex(val.as_str())
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Wrong encoding for checksum256 string: {}",
                            e
                        )))?;
                    Ok(c.pack(encoder))
                }
                "checksum512" => {
                    let c = Checksum512::from_hex(val.as_str())
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Wrong encoding for checksum512 string: {}",
                            e
                        )))?;
                    Ok(c.pack(encoder))
                }
                "public_key" => {
                    let key = PublicKey::new_from_str(val.as_str())
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Wrong encoding for public key string: {}",
                            e
                        )))?;
                    Ok(key.pack(encoder))
                }
                "signature" => {
                    let sig = Signature::from_string(val.as_str())
                        .map_err(|e| PyErr::new::<PyValueError, _>(format!(
                            "Wrong encoding for signature string: {}",
                            e
                        )))?;
                    Ok(sig.pack(encoder))
                }
                other => Err(PyErr::new::<PyValueError, _>(format!(
                    "Unexpected string type for field '{}'",
                    other
                ))),
            }
        }

        ActionDataTypes::List(py_list) => {
            // If we got here, it might be a variant (encoded as [type, value]),
            // because array handling was done earlier.
            let variant_meta = abi
                .resolve_type(field_type.as_str())
                .ok_or_else(|| PyErr::new::<PyValueError, _>(format!(
                    "Expected to find a variant type for '{}'",
                    field_type
                )))?;

            let variant_types = match variant_meta {
                ABIResolvedType::Variant(ref v) => v,
                ABIResolvedType::Struct(_) => {
                    return Err(PyErr::new::<PyValueError, _>(
                        "Expected a variant but got a struct"
                    ));
                }
            };

            let list = py_list.bind(py);
            if list.len() != 2 {
                return Err(PyErr::new::<PyValueError, _>(
                    "Expected variant encoded as list [type, value] of length 2"
                ));
            }

            let variant_type: String = list.get_item(0)?.extract()?;
            let variant_index = variant_types
                .types
                .iter()
                .position(|var_type_name| **var_type_name == variant_type)
                .ok_or_else(|| {
                    PyErr::new::<PyValueError, _>(format!(
                        "Variant type '{}' not found in variant definition",
                        variant_type
                    ))
                })?;

            size += VarUint32::new(variant_index as u32).pack(encoder);

            let variant_val: ActionDataTypes = list.get_item(1)?.extract()?;
            Ok(encode_abi_type(py, abi, &variant_type, &variant_val, encoder)?)
        }

        ActionDataTypes::Struct(py_dict) => {
            let dict = py_dict.bind(py);
            let resolved_type = abi
                .resolve_type(field_type.as_str())
                .ok_or_else(|| PyErr::new::<PyValueError, _>(format!(
                    "Expected to resolve type '{}' to a struct or variant",
                    field_type
                )))?;

            match resolved_type {
                ABIResolvedType::Struct(struct_meta) => {
                    let mut struct_size = 0;
                    for field in &struct_meta.fields {
                        let item = dict
                            .get_item(&field.name)?
                            .ok_or_else(|| PyErr::new::<PyKeyError, _>(format!(
                                "Missing field '{}' in struct",
                                field.name
                            )))?;

                        let val: ActionDataTypes = item.extract()?;
                        struct_size += encode_abi_type(py, abi, &field.r#type, &val, encoder)?;
                    }
                    return Ok(struct_size);
                }
                ABIResolvedType::Variant(_) => {
                    return Err(PyErr::new::<PyValueError, _>(
                        "Unexpected variant type where struct was expected"
                    ));
                }
            }
        }

        ActionDataTypes::Name(name) => {
            Ok(name.inner.pack(encoder))
        }

        ActionDataTypes::SymbolCode(sym_code) => {
            Ok(sym_code.inner.pack(encoder))
        }

        ActionDataTypes::Symbol(sym) => {
            Ok(sym.inner.pack(encoder))
        }

        ActionDataTypes::Asset(asset) => {
            Ok(asset.inner.pack(encoder))
        }

        other => {
            return Err(PyErr::new::<PyValueError, _>(format!(
                "Unexpected action data type: {:?}",
                other
            )));
        }
    }?;

    Ok(size)
}

pub fn encode_params(
    account_name: &str,
    action_name: &str,
    params: &Vec<ActionDataTypes>,
) -> PyResult<Vec<u8>> {
    let abis = ABIS.lock().unwrap();
    let abi = match abis.get(account_name) {
        Some(abi) => Ok(abi),
        None => Err(PyErr::new::<PyKeyError, _>(format!("ABI for account '{}' not found", account_name))),
    }?;
    let struct_meta: &AbiStruct = abi.structs.iter().find(|s| s.name == *action_name).unwrap();

    let mut size = 0;
    let mut encoder = Encoder::new(0);
    for (i, field_value) in params.iter().enumerate() {
        let field_name = struct_meta.fields.get(i).expect("Field not found").name.clone();

        let field_type: String = struct_meta.fields.iter().find(|f| f.name == field_name)
            .unwrap()
            .r#type.clone();

        if account_name == "eosio" && action_name == "setabi" && field_name == "abi" {
            let abi_str = match field_value {
                ActionDataTypes::Bytes(abi_bytes) => Ok(String::from_utf8(abi_bytes.clone())?),
                _ => Err(PyErr::new::<PyValueError, _>("Expected eosio::setabi::abi param to be of type bytes")),
            }?;
            let abi = ABI::from_string(&abi_str).map_err(|e| PyErr::new::<PyValueError, _>(e))?;
            size += abi.pack(&mut encoder);
        }

        size += Python::with_gil(|py| encode_abi_type(py, abi, &field_type, &field_value, &mut encoder))?;
    }
    let encoder_size = encoder.get_size();
    if size != encoder_size {
        return Err(PyErr::new::<PyValueError, _>(format!("Encoder size mismatch: {} != {}", size, encoder_size)));
    }
    Ok(encoder.get_bytes().to_vec())
}
