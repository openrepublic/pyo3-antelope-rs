pub mod proxies;
pub mod serializer;
pub mod sign;

use crate::proxies::abi::{ShipABI, ABI};
use crate::proxies::asset::ExtendedAsset;
use crate::proxies::checksums::{Checksum160, Checksum256, Checksum512};
use crate::proxies::private_key::PrivateKey;
use crate::proxies::public_key::PublicKey;
use crate::proxies::signature::Signature;
use crate::proxies::{asset::Asset, name::Name, sym::Symbol, sym_code::SymbolCode};
use crate::sign::sign_tx;
use antelope::chain::abi::BUILTIN_TYPES;
use pyo3::panic::PanicException;
use pyo3::prelude::*;
use pyo3::types::{PyFrozenSet, PyInt};

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
