use std::collections::HashMap;
use antelope::chain::abi::{AbiStruct, ABI};
use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::asset::{Asset, Symbol};
use antelope::chain::checksum::{Checksum160, Checksum256, Checksum512};
use antelope::chain::name::Name;
use antelope::chain::private_key::PrivateKey;
use antelope::chain::public_key::PublicKey;
use antelope::chain::signature::Signature;
use antelope::chain::time::TimePointSec;
use antelope::chain::transaction::{CompressionType, PackedTransaction, SignedTransaction, Transaction, TransactionHeader};
use antelope::chain::varint::VarUint32;
use antelope::serializer::{Encoder, Packer};
use antelope::util::bytes_to_hex;
use chrono::NaiveDateTime;
use pyo3::prelude::*;
use pyo3::PyNativeType;
use pyo3::types::{PyBytes, PyDict, PyFloat, PyList, PyLong};


pub fn str_to_timestamp(ts: &str) -> u32 {
    let naive_dt = NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S")
        .expect("Failed to parse datetime");

    naive_dt.and_utc().timestamp() as u32
}

#[derive(Debug)]
pub enum ActionDataTypes {
    Bool(bool),
    Int(Py<PyLong>),
    Float(Py<PyFloat>),
    Bytes(Vec<u8>),
    String(String),
    List(Py<PyList>),
    Struct(Py<PyDict>),
    None,
}

impl<'a> FromPyObject<'a> for ActionDataTypes {
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        // Check if it's None
        if ob.is_none() {
            return Ok(ActionDataTypes::None);
        }

        // Try downcasting to bool
        if let Ok(py_bool) = ob.extract::<bool>() {
            return Ok(ActionDataTypes::Bool(py_bool));
        }

        // Try downcasting to PyInt
        if let Ok(py_int) = ob.downcast::<PyLong>() {
            return Ok(ActionDataTypes::Int(py_int.into_py(ob.py())));
        }

        // Try downcasting to PyFloat
        if let Ok(py_float) = ob.downcast::<PyFloat>() {
            return Ok(ActionDataTypes::Float(py_float.into_py(ob.py())));
        }

        // Try extracting as bytes (Python `bytes` or `bytearray`).
        //    ob.extract::<Vec<u8>>() handles both.
        if let Ok(byte_vec) = ob.downcast::<PyBytes>() {
            let byte_vec: Vec<u8> = byte_vec.extract()?;
            return Ok(ActionDataTypes::Bytes(byte_vec));
        }

        // Try extracting as a String
        if let Ok(s) = ob.extract::<String>() {
            return Ok(ActionDataTypes::String(s));
        }

        // Check if it's a list
        if let Ok(py_list) = ob.downcast::<PyList>() {
            return Ok(ActionDataTypes::List(py_list.into_py(ob.py())));
        }

        // Check if it's a dict
        if let Ok(py_dict) = ob.downcast::<PyDict>() {
            return Ok(ActionDataTypes::Struct(py_dict.into_py(ob.py())));
        }

        // If all attempts failed, raise a TypeError:
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            format!("Cannot convert to ActionDataTypes: {:?}", ob)
        ))
    }
}

pub fn encode_abi_type(py: Python, abi: &ABI, field_type: &String, field_value: &ActionDataTypes, encoder: &mut Encoder) {
    let og_field_type = field_type.clone();
    let mut field_type = field_type.clone();
    // handle array
    if field_type.ends_with("[]") {
        // remove list type hint
        field_type.truncate(field_type.len().saturating_sub(2));
        match field_value {
            ActionDataTypes::List(py_list) => {
                let l: Vec<ActionDataTypes> = py_list.extract(py).unwrap();
                for value in l {
                    encode_abi_type(py, abi, &field_type, &value, encoder);
                }
                return;
            }
            _ => {
                panic!("Expected list value for field {}: {:?}", og_field_type, field_value)
            }
        }
    }

    // handle optional
    if field_type.ends_with("?") {
        // remove optional type hint
        field_type.pop();
        match field_value {
            ActionDataTypes::None => {
                0u8.pack(encoder);
                return;
            },
            _ => {
                1u8.pack(encoder);
            }
        }
    }

    match field_value {
        ActionDataTypes::Bool(val) => {
            val.pack(encoder);
        }
        ActionDataTypes::Int(val) => {
            match field_type.as_str() {
                "int8" => {
                    let val: i8 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "int16" => {
                    let val: i16 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "int32" => {
                    let val: i32 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "int64" => {
                    let val: i64 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "int128" => {
                    let val: i128 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "uint8" => {
                    let val: u8 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "uint16" => {
                    let val: u16 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "uint32" => {
                    let val: u32 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "uint64" => {
                    let val: u64 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "uint128" => {
                    let val: u128 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "varuint32" => {
                    let val: u32 = val.extract(py).unwrap();
                    let val = VarUint32::new(val);
                    val.pack(encoder);
                }

                "time_point_sec" => {
                    let val: u32 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "name" => {
                    let val: u64 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "account_name" => {
                    let val: u64 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                _ => panic!("Unexpected type for int field"),
            }
        }
        ActionDataTypes::Float(val) => {
            match field_type.as_str() {
                "float32" => {
                    let val: f32 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                "float64" => {
                    let val: f64 = val.extract(py).unwrap();
                    val.pack(encoder);
                }
                _ => panic!("Unexpected type for float32 field"),
            }
        }
        ActionDataTypes::Bytes(val) => {
            val.pack(encoder);
        }
        ActionDataTypes::String(val) => {
            match field_type.as_str() {
                "string" => {
                    val.pack(encoder);
                }
                "time_point_sec" => {
                    let ts = str_to_timestamp(val.as_str());
                    ts.pack(encoder);
                }
                "name" => {
                    Name::new_from_str(val).pack(encoder);
                }
                "account_name" => {
                    Name::new_from_str(val).pack(encoder);
                }
                "symbol" => {
                    let (first, name) = val.split_once(",").unwrap();
                    let precision = first.parse::<u8>().expect("Wrong encoding for symbol precision");
                    Symbol::new(&name, precision).pack(encoder);
                }
                "asset" => {
                    Asset::from_string(val).pack(encoder);
                }
                "checksum160" | "rd160" => {
                    Checksum160::from_hex(val.as_str())
                        .expect("Wrong encoding for checksum160 string")
                        .pack(encoder);
                }
                "checksum256" | "sha256" => {
                    Checksum256::from_hex(val.as_str())
                        .expect("Wrong encoding for checksum256 string")
                        .pack(encoder);
                }
                "checksum512" => {
                    Checksum512::from_hex(val.as_str())
                        .expect("Wrong encoding for checksum512 string")
                        .pack(encoder);
                }
                "public_key" => {
                    PublicKey::new_from_str(val.as_str())
                        .expect("Wrong encoding for public key string")
                        .pack(encoder);
                }
                "signature" => {
                    Signature::from_string(val.as_str())
                        .expect("Wrong encoding for signature string")
                        .pack(encoder);
                }
                _ => panic!("Unexpected type for string field"),
            }
        }
        ActionDataTypes::Struct(dict) => {
            let obj = dict.into_py(py);
            let dict = obj.downcast::<PyDict>(py).unwrap();
            match field_type.as_str() {
                "variant" => {
                    let vname = dict.get_item("name").expect("Missing name field")
                        .unwrap().extract::<String>().unwrap();
                    vname.pack(encoder);
                    let types = dict.get_item("types").expect("Missing types field").unwrap().extract::<Vec<String>>().unwrap();
                    types.pack(encoder);
                }
                _ => {
                    let struct_meta: &AbiStruct = abi.structs.iter().find(|s| s.name == field_type).unwrap();
                    for field in &struct_meta.fields {
                        let val: ActionDataTypes = dict
                            .get_item(field.name.clone())
                            .expect("Missing field")
                            .unwrap()
                            .extract()
                            .unwrap();
                        encode_abi_type(py, abi, &field.r#type, &val, encoder);
                    }
                }
            }
        }
        _ => panic!("Unexpected action data type at this stage of encode_struct"),
    }
}

pub fn params_to_encoded_action_data(py: Python, action_name: &String, params: &Vec<ActionDataTypes>, abi: &ABI) -> PyResult<Vec<u8>> {
    let struct_meta: &AbiStruct = abi.structs.iter().find(|s| s.name == *action_name).unwrap();

    let mut encoder = Encoder::new(0);
    for (i, field_value) in params.iter().enumerate() {
        let field_name = struct_meta.fields.get(i).expect("Field not found").name.clone();

        let field_type: String = struct_meta.fields.iter().find(|f| f.name == field_name)
            .unwrap()
            .r#type.clone();

        encode_abi_type(py, abi, &field_type, &field_value, &mut encoder);
    }
    Ok(encoder.get_bytes().to_vec())
}

pub struct PyAction {
    account: String,
    name: String,
    authorization: Vec<PermissionLevel>,
    data: Vec<ActionDataTypes>,
}

fn convert_to_action(py: Python, action: &PyAction, abi: &ABI) -> PyResult<Action> {
    Ok(Action {
        account: Name::new_from_str(&action.account),
        name: Name::new_from_str(&action.name),
        authorization: action.authorization.clone(),
        data: params_to_encoded_action_data(py, &action.name, &action.data, abi)?,
    })
}

impl<'a> FromPyObject<'a> for PyAction {
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        if let Ok(py_dict) = ob.downcast::<PyDict>() {
            let account: String = py_dict
                .get_item("account")
                .expect("No account in Python object")
                .unwrap()
                .extract()
                .expect("account not a string");

            let name: String = py_dict
                .get_item("name")
                .expect("No name in Python object")
                .unwrap()
                .extract()
                .expect("name not a string");

            let data: Vec<ActionDataTypes> = py_dict
                .get_item("data")
                .expect("No data in Python object")
                .unwrap()
                .extract()
                .expect("data not a Vec<ActionDataTypes>");

            // "authorization": list of dicts -> Vec<PermissionLevel>
            let auth_list = py_dict
                .get_item("authorization")
                .unwrap()
                .unwrap()
                .extract::<Vec<&PyDict>>()?;

            let mut authorization = Vec::with_capacity(auth_list.len());
            for auth_dict in auth_list {
                // "actor"
                let actor_str = auth_dict
                    .get_item("actor")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()?;
                let actor = Name::new_from_str(&actor_str);

                // "permission"
                let perm_str = auth_dict
                    .get_item("permission")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()?;
                let permission = Name::new_from_str(&perm_str);

                authorization.push(PermissionLevel { actor, permission });
            }

            return Ok(PyAction {
                account, name, authorization, data
            });
        }
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Error in Python object supplied"))
    }
}

#[pyfunction]
#[pyo3(pass_module)]
fn create_and_sign_tx(
    m: &PyModule,
    chain_id: Vec<u8>,
    abis: HashMap<String, String>,
    actions: Vec<PyAction>,
    key: &str,
    expiration: u32,
    max_cpu_usage_ms: u8,
    max_net_usage_words: u32,
    ref_block_num: u16,
    ref_block_prefix: u32
) -> PyResult<Py<PyDict>> {
    let private_key = PrivateKey::from_str(key, false)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid key format! {e}")))?;

    let header = TransactionHeader {
        expiration: TimePointSec::new(expiration),
        ref_block_num,
        ref_block_prefix,
        max_net_usage_words: VarUint32::new(max_net_usage_words),
        max_cpu_usage_ms,
        delay_sec: VarUint32::new(0),
    };

    let actions = actions.iter().map(|a| {
        let account = a.account.clone();
        let abi = ABI::from_string(
            abis.get(&account).expect(format!("ABI for {account} not found!").as_str())
        ).expect("Invalid ABI");
        convert_to_action(m.py(), a, &abi).unwrap()
    }).collect();

    let transaction = Transaction {
        header,
        context_free_actions: vec![],
        actions,
        extension: vec![],
    };

    let sign_data = transaction.signing_data(chain_id.as_slice());

    let signed_tx = SignedTransaction {
        transaction,
        signatures: vec![private_key.sign_message(&sign_data)],
        context_free_data: vec![]
    };

    let tx = PackedTransaction::from_signed(signed_tx, CompressionType::NONE).unwrap();

    let dict_tx = PyDict::new(m.py());

    let signatures: Vec<String> = tx.signatures.iter().map(|s| s.to_string()).collect();
    let context_free_data: Vec<u8> = Vec::new();
    let packed_trx: String = bytes_to_hex(&tx.packed_transaction);


    dict_tx.set_item("signatures", signatures)?;
    dict_tx.set_item("compression", false)?;
    dict_tx.set_item("packed_context_free_data", context_free_data)?;
    dict_tx.set_item("packed_trx", packed_trx)?;

    Ok(dict_tx.into_py(m.py()))
}

#[pymodule]
fn antelope_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_and_sign_tx, m)?)?;

    Ok(())
}