use antelope::chain::abi::{ABIResolvedType, ABI};
use antelope::chain::checksum::{Checksum160, Checksum256, Checksum512};
use antelope::chain::asset::{
    Asset as NativeAsset,
    ExtendedAsset,
    Symbol as NativeSymbol,
    SymbolCode as NativeSymbolCode,
};
use antelope::chain::name::Name as NativeName;
use antelope::chain::Decoder;
use antelope::chain::public_key::PublicKey;
use antelope::chain::signature::Signature;
use antelope::chain::time::{BlockTimestamp, TimePoint, TimePointSec};
use antelope::chain::varint::VarUint32;
use pyo3::{IntoPyObject, Py, PyResult, Python};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyInt, PyList};
use crate::proxies::asset::Asset;
use crate::proxies::name::Name;
use crate::proxies::sym::Symbol;
use crate::proxies::sym_code::SymbolCode;
use crate::types::ActionDataTypes;
use crate::utils::{timestamp_ms_to_str, timestamp_to_str};

pub fn decode_abi_type(
    py: Python,
    abi: &ABI,
    field_type: &str,
    buf_size: usize,
    decoder: &mut Decoder
) -> PyResult<ActionDataTypes> {
    let (field_meta, resolved_type) = match abi.resolve_type(&field_type) {
        Some(val) => Ok(val),
        None => Err(PyTypeError::new_err(format!("{} not found in ABI", field_type))),
    }?;

    match field_meta {
        ABIResolvedType::Standard(std_type) => {
            match std_type.as_str() {
                "bool" => {
                    let mut val = 0u8;
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::Bool(val == 1u8))
                }
                "int8" => {
                    let mut val = 0i8;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "int16" => {
                    let mut val = 0i16;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "int32" => {
                    let mut val = 0i32;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "int64" => {
                    let mut val = 0i64;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "int128" => {
                    let mut val = 0i128;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "uint8" => {
                    let mut val = 0u8;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "uint16" => {
                    let mut val = 0u16;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "uint32" => {
                    let mut val = 0u32;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "uint64" => {
                    let mut val = 0u64;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "uint128" => {
                    let mut val = 0u128;
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "varuint32" => {
                    let mut val = VarUint32::default();
                    decoder.unpack(&mut val);

                    let py_int: Py<PyInt> = val.n.into_pyobject(py)?
                        .downcast::<PyInt>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Int(py_int))
                }
                "float32" => {
                    let mut val = 0f32;
                    decoder.unpack(&mut val);

                    let py_float: Py<PyFloat> = val.into_pyobject(py)?
                        .downcast::<PyFloat>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Float(py_float))
                }
                "float64" => {
                    let mut val = 0f64;
                    decoder.unpack(&mut val);

                    let py_float: Py<PyFloat> = val.into_pyobject(py)?
                        .downcast::<PyFloat>()?
                        .clone()
                        .unbind();

                    Ok(ActionDataTypes::Float(py_float))
                }
                "bytes" => {
                    let mut val: Vec<u8> = Vec::new();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::Bytes(val))
                }
                "string" => {
                    let mut val = String::new();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::String(val))
                }
                "rd160" | "checksum160" => {
                    let mut val = Checksum160::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::String(val.to_string()))
                }
                "sha256" | "checksum256" | "transaction_id" => {
                    let mut val = Checksum256::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::String(val.to_string()))
                }
                "checksum512" => {
                    let mut val = Checksum512::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::String(val.to_string()))
                }
                "name" | "account_name" => {
                    let mut val = NativeName::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::Name(Name { inner: val }))
                }
                "symbol_code" => {
                    let mut val = NativeSymbolCode::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::SymbolCode(SymbolCode { inner: val }))
                }
                "symbol" => {
                    let mut val = NativeSymbol::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::Symbol(Symbol { inner: val }))
                }
                "asset" => {
                    let mut val = NativeAsset::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::Asset(Asset { inner: val }))
                }
                "extended_asset" => {
                    let mut val = ExtendedAsset::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::String(val.to_string()))
                }
                "public_key" => {
                    let mut val = PublicKey::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::String(val.to_string()))
                }
                "signature" => {
                    let mut val = Signature::default();
                    decoder.unpack(&mut val);

                    Ok(ActionDataTypes::String(val.to_string()))
                }
                "block_timestamp_type" => {
                    let mut val = BlockTimestamp::default();
                    decoder.unpack(&mut val);

                    let tp = val.to_time_point_sec();

                    let time_str = match timestamp_to_str(tp.seconds) {
                        Some(val) => Ok(val),
                        None => Err(PyTypeError::new_err(format!("Could not convert {} to timestamp string", tp.seconds)))
                    }?;

                    Ok(ActionDataTypes::String(time_str))
                }
                "time_point_sec" => {
                    let mut val = TimePointSec::default();
                    decoder.unpack(&mut val);

                    let time_str = match timestamp_to_str(val.seconds) {
                        Some(val) => Ok(val),
                        None => Err(PyTypeError::new_err(format!("Could not convert {} to timestamp string", val.seconds)))
                    }?;

                    Ok(ActionDataTypes::String(time_str))
                }
                "time_point" => {
                    let mut val = TimePoint::default();
                    decoder.unpack(&mut val);

                    let time_str = match timestamp_ms_to_str(val.elapsed) {
                        Some(val) => Ok(val),
                        None => Err(PyTypeError::new_err(format!("Could not convert {} to timestamp ms string", val.elapsed)))
                    }?;

                    Ok(ActionDataTypes::String(time_str))
                }
                _ => Err(PyValueError::new_err(format!("Unknown standard type {}", field_type)))
            }
        }
        ABIResolvedType::Optional(_) => {
            let mut flag: u8 = 0;
            decoder.unpack(&mut flag);

            if flag == 1 {
                decode_abi_type(py, abi, &resolved_type, buf_size, decoder)
            } else {
                Ok(ActionDataTypes::None)
            }
        }
        ABIResolvedType::Array(_) => {
            let mut len = VarUint32::new(0);
            decoder.unpack(&mut len);

            let py_list = PyList::empty(py);
            for _ in 0..len.n {
                let result = decode_abi_type(py, abi, &resolved_type, buf_size, decoder)?;
                py_list.append(result)?;
            }
            Ok(ActionDataTypes::List(py_list.unbind()))
        }
        ABIResolvedType::Extension(_) => {
            if decoder.get_pos() < buf_size {
                let result = decode_abi_type(py, abi, &resolved_type, buf_size, decoder)?;
                return Ok(result);
            }
            Ok(ActionDataTypes::None)
        }
        ABIResolvedType::Variant(inner) => {
            let mut vindex = VarUint32::new(0);
            decoder.unpack(&mut vindex);

            let var_type: String = match inner.types.get(vindex.n as usize) {
                Some(var_type) => Ok(var_type.clone()),
                None => Err(PyValueError::new_err(format!("Variant {} does not have type at index {}", inner.name, vindex.n))),
            }?;

            let py_list = PyList::empty(py);
            py_list.append(var_type.clone())?;
            py_list.append(decode_abi_type(py, abi, &var_type, buf_size, decoder)?)?;
            Ok(ActionDataTypes::List(py_list.unbind()))
        }
        ABIResolvedType::Struct(inner) => {
            let result_dict = PyDict::new(py);
            for field in &inner.fields {
                let result = decode_abi_type(py, abi, &field.r#type, buf_size, decoder)?;
                result_dict.set_item(field.name.clone(), result)?;
            }
            Ok(ActionDataTypes::Struct(result_dict.unbind()))
        }
    }
}
