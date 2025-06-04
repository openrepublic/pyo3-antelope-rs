use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::name::Name as NativeName;
use antelope::chain::time::TimePointSec;
use antelope::chain::transaction::TransactionHeader;
use antelope::chain::transaction::{
    CompressionType, PackedTransaction, SignedTransaction, Transaction,
};
use antelope::chain::varint::VarUint32;
use antelope::util::bytes_to_hex;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyDict;
use pyo3::{pyfunction, FromPyObject, PyResult};
use std::str::FromStr;

use crate::proxies::checksums::{PyChecksum256, Sum256Like};
use crate::proxies::private_key::PyPrivateKey;
use pyo3::prelude::*;

#[derive(FromPyObject)]
pub struct PyPermissionLevel {
    actor: String,
    permission: String,
}

impl From<&PyPermissionLevel> for PyResult<PermissionLevel> {
    fn from(value: &PyPermissionLevel) -> Self {
        Ok(PermissionLevel::new(
            NativeName::from_str(&value.actor).map_err(|e| PyValueError::new_err(e.to_string()))?,
            NativeName::from_str(&value.permission)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
}

#[derive(FromPyObject)]
pub struct PyAction {
    account: String,
    name: String,
    authorization: Vec<PyPermissionLevel>,
    data: Vec<u8>,
}

impl From<&PyAction> for PyResult<Action> {
    fn from(py_action: &PyAction) -> Self {
        let mut auths = Vec::new();
        for auth in py_action.authorization.iter() {
            let maybe_perm: PyResult<PermissionLevel> = auth.into();
            auths.push(maybe_perm?);
        }
        Ok(Action {
            account: NativeName::from_str(&py_action.account)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
            name: NativeName::from_str(&py_action.name)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
            authorization: auths,
            data: py_action.data.clone(),
        })
    }
}

#[derive(FromPyObject)]
pub struct PyTransactionHeader {
    pub expiration: u32,
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub max_net_usage_words: u32,
    pub max_cpu_usage_ms: u8,
    pub delay_sec: u32,
}

impl From<PyTransactionHeader> for TransactionHeader {
    fn from(value: PyTransactionHeader) -> Self {
        TransactionHeader {
            expiration: TimePointSec::new(value.expiration),
            ref_block_num: value.ref_block_num,
            ref_block_prefix: value.ref_block_prefix,
            max_net_usage_words: VarUint32::new(value.max_net_usage_words),
            max_cpu_usage_ms: value.max_cpu_usage_ms,
            delay_sec: VarUint32::new(value.delay_sec),
        }
    }
}

#[pyfunction]
pub fn sign_tx(
    chain_id: Sum256Like,
    header: PyTransactionHeader,
    actions: Vec<PyAction>,
    sign_key: &PyPrivateKey,
) -> PyResult<Py<PyDict>> {
    // convert to valid checksum256
    let chain_id = PyChecksum256::try_from(chain_id)?;

    // convert py actions into native
    let mut _actions: Vec<Action> = Vec::with_capacity(actions.len());
    for action in actions.iter() {
        let act: PyResult<Action> = action.into();
        _actions.push(act?);
    }
    let actions: Vec<Action> = _actions;

    // put together transaction to sign
    let transaction = Transaction {
        header: header.into(),
        context_free_actions: vec![],
        actions,
        extension: vec![],
    };

    // sign using chain id
    let sign_data = transaction.signing_data(chain_id.raw());
    let signed_tx = SignedTransaction {
        transaction,
        signatures: vec![sign_key
            .inner
            .sign_message(&sign_data)
            .map_err(|e| PyValueError::new_err(e.to_string()))?],
        context_free_data: vec![],
    };

    // finally PackedTransaction is the payload to be broadcasted
    let tx = PackedTransaction::from_signed(signed_tx, CompressionType::NONE)
        .map_err(|e| PyValueError::new_err(format!("Error signing packed trx: {e}")))?;

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
