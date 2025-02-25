pub mod proxies;

pub mod types;
pub mod encode;
pub mod utils;
pub mod abi_store;

use antelope::chain::action::Action;
use antelope::chain::key_type::{KeyType, KeyTypeTrait};
use antelope::chain::private_key::PrivateKey;
use antelope::chain::time::TimePointSec;
use antelope::chain::transaction::{CompressionType, PackedTransaction, SignedTransaction, Transaction, TransactionHeader};
use antelope::chain::varint::VarUint32;
use antelope::util::bytes_to_hex;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::abi_store::{load_abi, unload_abi};
use crate::proxies::{
    name::Name,
    sym_code::SymbolCode,
    sym::Symbol,
    asset::Asset,
};
use crate::types::{into_action, PyAction};

#[pyfunction]
fn create_and_sign_tx(
    chain_id: Vec<u8>,
    actions: Vec<PyAction>,
    key: &str,
    expiration: u32,
    max_cpu_usage_ms: u8,
    max_net_usage_words: u32,
    ref_block_num: u16,
    ref_block_prefix: u32
) -> PyResult<Py<PyDict>> {
    // unpack key string into PrivateKey struct
    let private_key = PrivateKey::from_str(key, false)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid key format! {e}")))?;

    let header = TransactionHeader {
        expiration: TimePointSec::new(expiration),
        ref_block_num,
        ref_block_prefix,
        max_net_usage_words: VarUint32::new(max_net_usage_words),
        max_cpu_usage_ms,
        delay_sec: VarUint32::new(0),
    };

    // PyAction contains the un-serialized action params
    // converting it to an antelope::chain::Action requires
    // serializing the action params
    let mut _actions: Vec<Action> = Vec::new();
    for action in actions {
        _actions.push(into_action(&action)?);
    }
    let actions = _actions;

    // put together transaction to sign
    let transaction = Transaction {
        header,
        context_free_actions: vec![],
        actions,
        extension: vec![],
    };

    // sign using chain id
    let sign_data = transaction.signing_data(chain_id.as_slice());
    let signed_tx = SignedTransaction {
        transaction,
        signatures: vec![private_key.sign_message(&sign_data)],
        context_free_data: vec![]
    };

    // finally PackedTransaction is the payload to be broadcasted
    let tx = PackedTransaction::from_signed(signed_tx, CompressionType::NONE).unwrap();

    // pack and return into a bounded PyDict
    Python::with_gil(|py| {
        let dict_tx = PyDict::new(py);

        let signatures: Vec<String> = tx.signatures.iter().map(|s| s.to_string()).collect();
        let packed_trx: String = bytes_to_hex(&tx.packed_transaction);


        dict_tx.set_item("signatures", signatures)?;
        dict_tx.set_item("compression", false)?;
        dict_tx.set_item("packed_context_free_data", "".to_string())?;
        dict_tx.set_item("packed_trx", packed_trx)?;

        Ok(dict_tx.unbind())
    })
}

#[pyfunction]
fn gen_key_pair(key_type: u8) -> PyResult<(String, String)> {
    let key_type = KeyType::from_index(key_type)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid key type format {}", e)))?;

    let private_key = PrivateKey::random(key_type)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid key format {}", e)))?;

    Ok((private_key.as_string(), private_key.to_public().to_string()))
}

#[pyfunction]
fn get_pub_key(private_key: String) -> PyResult<String> {
    let private_key = PrivateKey::from_str(private_key.as_str(), false)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid key format {}", e)))?;

    Ok(private_key.to_public().to_string())
}

#[pymodule]
fn antelope_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    // abi store mngmnt
    m.add_function(wrap_pyfunction!(load_abi, m)?)?;
    m.add_function(wrap_pyfunction!(unload_abi, m)?)?;

    // utilities
    m.add_function(wrap_pyfunction!(gen_key_pair, m)?)?;
    m.add_function(wrap_pyfunction!(get_pub_key, m)?)?;

    // tx pack
    m.add_function(wrap_pyfunction!(create_and_sign_tx, m)?)?;

    // proxy classes
    m.add_class::<Name>()?;
    m.add_class::<SymbolCode>()?;
    m.add_class::<Symbol>()?;
    m.add_class::<Asset>()?;

    Ok(())
}