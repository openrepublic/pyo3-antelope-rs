use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use antelope::chain::abi::ABI;
use pyo3::{pyfunction, PyResult};
use pyo3::exceptions::PyValueError;

pub static ABIS: LazyLock<Mutex<HashMap<String, ABI>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

#[pyfunction]
pub fn load_abi(account: &str, abi: &str) -> PyResult<()> {
    let mut abis = ABIS.lock().unwrap();
    let abi = ABI::from_string(abi).map_err(|_| PyValueError::new_err("Invalid ABI"))?;
    abis.insert(account.to_string(), abi);
    Ok(())
}

#[pyfunction]
pub fn unload_abi(account: &str) -> PyResult<()> {
    let mut abis = ABIS.lock().unwrap();
    abis.remove(account);
    Ok(())
}
