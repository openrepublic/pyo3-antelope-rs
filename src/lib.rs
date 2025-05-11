pub mod proxies;

pub mod types;
pub mod utils;

use std::collections::HashMap;
use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::time::TimePointSec;
use antelope::chain::transaction::{CompressionType, PackedTransaction, SignedTransaction, Transaction, TransactionHeader};
use antelope::chain::varint::VarUint32;
use antelope::util::bytes_to_hex;
use pyo3::exceptions::{PyValueError};
use pyo3::panic::PanicException;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::proxies::{
    name::Name,
    sym_code::SymbolCode,
    sym::Symbol,
    asset::Asset,
};
use crate::proxies::abi::{ShipABI, ABI};
use crate::proxies::asset::ExtendedAsset;
use crate::proxies::checksums::{Checksum160, Checksum256, Checksum512};
use crate::proxies::private_key::PrivateKey;
use crate::proxies::public_key::PublicKey;
use crate::proxies::signature::Signature;
use crate::types::AntelopeValue;

#[pyfunction]
fn create_and_sign_tx(
    chain_id: Vec<u8>,
    actions: AntelopeValue,
    mut abis: HashMap<Name, ABI>,
    sign_key: &PrivateKey,
    expiration: u32,
    max_cpu_usage_ms: u8,
    max_net_usage_words: u32,
    ref_block_num: u16,
    ref_block_prefix: u32
) -> PyResult<Py<PyDict>> {
    let header = TransactionHeader {
        expiration: TimePointSec::new(expiration),
        ref_block_num,
        ref_block_prefix,
        max_net_usage_words: VarUint32::new(max_net_usage_words),
        max_cpu_usage_ms,
        delay_sec: VarUint32::new(0),
    };

    let actions: Vec<HashMap<String, AntelopeValue>> = if let AntelopeValue::List(a) = actions {
        let mut _actions = Vec::new();
        for val in a {
            if let AntelopeValue::Dict(action) = val {
                _actions.push(action);
            } else {
                return Err(PyValueError::new_err(format!("Expected action item to be a Dict: {:?}", val)))
            }
        }
        Ok(_actions)
    } else {
        Err(PyValueError::new_err(format!("Expected action param to be a List: {:?}", actions)))
    }?;

    // serialize the action params
    let mut _actions: Vec<Action> = Vec::new();
    for action in actions {
        let account: Name = action.get("account")
            .ok_or(PyValueError::new_err("Action in action missing account key"))?
            .try_into()?;

        let name: Name = action.get("name")
            .ok_or(PyValueError::new_err("Action in action missing name key"))?
            .try_into()?;

        let data = action.get("data")
            .ok_or(PyValueError::new_err("Action in action missing name key"))?
            .clone();

        let authorization: Vec<PermissionLevel> = if let AntelopeValue::List(auths) = action.get("authorization")
            .ok_or(PyValueError::new_err("Action in map missing authorization key"))? {
            let mut _auths = Vec::new();
            for val in auths {
                if let AntelopeValue::Dict(auth) = val {
                    let actor: Name = auth.get("actor")
                        .ok_or(PyValueError::new_err(format!("Authorization missing actor key: {:?}", val)))?
                        .try_into()?;

                    let permission: Name = auth.get("permission")
                        .ok_or(PyValueError::new_err(format!("Authorization missing actor key: {:?}", val)))?
                        .try_into()?;

                    _auths.push(PermissionLevel{actor: actor.inner, permission: permission.inner})
                }
            }
            Ok(_auths)
        } else {
            Err(PyValueError::new_err(format!("Action in action missing authorization key")))
        }?;

        let abi = abis.get_mut(&account)
            .ok_or(PyValueError::new_err("Action in map missing ABI"))?;

        let packed_data = abi.pack(&name.to_string(), data)?;

        _actions.push(Action {
            account: account.inner,
            name: name.inner,
            data: packed_data,
            authorization
        });
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
        signatures: vec![sign_key.inner.sign_message(&sign_data)],
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

#[pymodule]
fn antelope_rs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    // pack/unpack
    m.add_function(wrap_pyfunction!(create_and_sign_tx, m)?)?;

    // proxy classes
    m.add_class::<Name>()?;

    m.add_class::<PrivateKey>()?;
    m.add_class::<PublicKey>()?;
    m.add_class::<Signature>()?;

    m.add_class::<Checksum160>()?;
    m.add_class::<Checksum256>()?;
    m.add_class::<Checksum512>()?;

    m.add_class::<SymbolCode>()?;
    m.add_class::<Symbol>()?;
    m.add_class::<Asset>()?;
    m.add_class::<ExtendedAsset>()?;

    m.add_class::<ABI>()?;
    m.add_class::<ShipABI>()?;

    m.add("PanicException", py.get_type::<PanicException>())?;

    Ok(())
}