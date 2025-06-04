pub mod serializer;
pub mod proxies;
pub mod types;

use antelope::chain::action::Action;
use antelope::chain::transaction::{CompressionType, PackedTransaction, SignedTransaction, Transaction};
use antelope::chain::abi::BUILTIN_TYPES;
use antelope::util::bytes_to_hex;
use pyo3::exceptions::PyValueError;
use pyo3::panic::PanicException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PyInt};
use types::PyTransactionHeader;
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
use crate::types::PyAction;

#[pyfunction]
fn sign_tx(
    chain_id: Vec<u8>,
    header: PyTransactionHeader,
    actions: Vec<PyAction>,
    sign_key: &PrivateKey,
) -> PyResult<Py<PyDict>> {

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
    let sign_data = transaction.signing_data(chain_id.as_slice());
    let signed_tx = SignedTransaction {
        transaction,
        signatures: vec![
            sign_key.inner.sign_message(&sign_data)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
        ],
        context_free_data: vec![]
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

#[pymodule(name="_lowlevel")]
fn antelope_rs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    let py_builtin_types = PyFrozenSet::new(
        py,
        BUILTIN_TYPES.iter()
    )?;
    m.add("builtin_types", py_builtin_types)?;

    let py_asset_max_amount = PyInt::new(
        py, antelope::chain::asset::ASSET_MAX_AMOUNT
    );
    m.add("asset_max_amount", py_asset_max_amount)?;

    let py_asset_max_precision = PyInt::new(
        py, antelope::chain::asset::ASSET_MAX_PRECISION
    );
    m.add("asset_max_precision", py_asset_max_precision)?;

    // pack/unpack
    m.add_function(wrap_pyfunction!(sign_tx, m)?)?;

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
