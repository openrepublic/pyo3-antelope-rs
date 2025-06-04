use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::name::Name as NativeName;
use antelope::chain::time::TimePointSec;
use antelope::chain::transaction::TransactionHeader;
use antelope::chain::varint::VarUint32;
use pyo3::{FromPyObject, PyResult};
use pyo3::exceptions::PyValueError;
use std::str::FromStr;

#[derive(FromPyObject)]
pub(crate) struct PyPermissionLevel {
    actor: String,
    permission: String
}

impl From<&PyPermissionLevel> for PyResult<PermissionLevel> {
    fn from(value: &PyPermissionLevel) -> Self {
        Ok(PermissionLevel::new(
            NativeName::from_str(&value.actor).map_err(|e| PyValueError::new_err(e.to_string()))?,
            NativeName::from_str(&value.permission).map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
}

#[derive(FromPyObject)]
pub(crate) struct PyAction {
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
            account: NativeName::from_str(&py_action.account).map_err(|e| PyValueError::new_err(e.to_string()))?,
            name: NativeName::from_str(&py_action.name).map_err(|e| PyValueError::new_err(e.to_string()))?,
            authorization: auths,
            data: py_action.data.clone(),
        })
    }
}

#[derive(FromPyObject)]
pub(crate) struct PyTransactionHeader {
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
            delay_sec: VarUint32::new(value.delay_sec)
        }
    }
}
