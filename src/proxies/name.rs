use antelope::chain::name::{Name as NativeName, NameError};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;

/// A Python-exposed wrapper around the `antelope::Name` struct.
#[pyclass]
pub struct Name {
    pub inner: NativeName,
}

#[pymethods]
impl Name {
    #[staticmethod]
    fn from_int(value: u64) -> PyResult<Self> {
        // If you'd like to mirror the original assertion, handle it as an error:
        let name = NativeName::from_u64(value);
        Ok(Name { inner: name })
    }

    #[staticmethod]
    fn from_str(s: &str) -> PyResult<Self> {
        match NativeName::from_string(s) {
            Ok(name) => Ok(Name { inner: name }),
            Err(NameError::InvalidName) => {
                Err(pyo3::exceptions::PyValueError::new_err("Invalid name string"))
            }
        }
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __int__(&self) -> u64 {
        self.inner.n
    }

    fn __richcmp__(&self, other: &Name, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}