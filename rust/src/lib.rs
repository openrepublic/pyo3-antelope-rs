pub mod proxies;
pub mod serializer;
pub mod sign;
mod utils;

use crate::proxies::abi::{PyShipABI, PyABI};
use crate::proxies::checksums::{PyChecksum160, PyChecksum256, PyChecksum512};
use crate::proxies::private_key::PyPrivateKey;
use crate::proxies::public_key::PyPublicKey;
use crate::proxies::signature::PySignature;
use crate::proxies::{asset::{PyAsset, PyExtendedAsset}, name::PyName, sym::PySymbol, sym_code::PySymbolCode};
use crate::sign::sign_tx;
use antelope::chain::abi::BUILTIN_TYPES;
use proxies::float::PyFloat128;
use proxies::time::{PyBlockTimestamp, PyTimePoint, PyTimePointSec};
use proxies::varint::{PyVarInt32, PyVarUInt32};
use proxies::TryFromError;
use pyo3::panic::PanicException;
use pyo3::prelude::*;
use pyo3::types::{PyFrozenSet, PyInt};
use utils::BytesStringDecodeError;

#[pymodule(name = "_lowlevel")]
fn antelope_rs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    let py_builtin_types = PyFrozenSet::new(py, BUILTIN_TYPES.iter())?;
    m.add("builtin_types", py_builtin_types)?;

    let py_asset_max_amount = PyInt::new(py, antelope::chain::asset::ASSET_MAX_AMOUNT);
    m.add("asset_max_amount", py_asset_max_amount)?;

    let py_asset_max_precision = PyInt::new(py, antelope::chain::asset::ASSET_MAX_PRECISION);
    m.add("asset_max_precision", py_asset_max_precision)?;

    // tx sign helper
    m.add_function(wrap_pyfunction!(sign_tx, m)?)?;

    // proxy classes
    m.add_class::<PyFloat128>()?;
    m.add_class::<PyVarUInt32>()?;
    m.add_class::<PyVarInt32>()?;
    m.add_class::<PyName>()?;

    m.add_class::<PyPrivateKey>()?;
    m.add_class::<PyPublicKey>()?;
    m.add_class::<PySignature>()?;

    m.add_class::<PyChecksum160>()?;
    m.add_class::<PyChecksum256>()?;
    m.add_class::<PyChecksum512>()?;

    m.add_class::<PySymbolCode>()?;
    m.add_class::<PySymbol>()?;
    m.add_class::<PyAsset>()?;
    m.add_class::<PyExtendedAsset>()?;

    m.add_class::<PyTimePoint>()?;
    m.add_class::<PyTimePointSec>()?;
    m.add_class::<PyBlockTimestamp>()?;

    m.add_class::<PyABI>()?;
    m.add_class::<PyShipABI>()?;

    m.add("PanicException", py.get_type::<PanicException>())?;
    m.add("TryFromError", py.get_type::<TryFromError>())?;
    m.add("BytesStringDecodeError", py.get_type::<BytesStringDecodeError>())?;

    Ok(())
}
