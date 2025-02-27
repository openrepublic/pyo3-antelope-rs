use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use antelope::chain::abi::ABI;
use pyo3::{pyfunction, PyErr, PyResult};
use pyo3::exceptions::{PyKeyError, PyTypeError, PyValueError};

pub static ABIS: LazyLock<Mutex<HashMap<String, ABI>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn get_abi(account: &str) -> PyResult<ABI> {
    let abis = ABIS.lock().unwrap();
    match abis.get(account) {
        Some(abi) => Ok(abi.clone()),
        None => Err(PyErr::new::<PyKeyError, _>(format!("ABI for account '{}' not found", account))),
    }
}

#[pyfunction]
pub fn load_abi(account: &str, abi: Vec<u8>) -> PyResult<()> {
    let mut abis = ABIS.lock().unwrap();
    let abi = ABI::from_string(
        &String::from_utf8(abi)
            .map_err(|_| PyTypeError::new_err("Could not decode buffer as utf-8 ABI"))?
    ).map_err(|_| PyValueError::new_err("Invalid ABI"))?;
    abis.insert(account.to_string(), abi);
    Ok(())
}

#[pyfunction]
pub fn unload_abi(account: &str) -> PyResult<()> {
    let mut abis = ABIS.lock().unwrap();
    abis.remove(account);
    Ok(())
}
