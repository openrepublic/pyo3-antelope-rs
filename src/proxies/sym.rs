use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use antelope::chain::asset::{Symbol as NativeSymbol, SymbolError};
use crate::proxies::sym_code::SymbolCode;

#[pyclass]
#[derive(Clone)]
pub struct Symbol {
    pub inner: NativeSymbol,
}

#[pymethods]
impl Symbol {

    #[staticmethod]
    fn from_str(s: &str) -> PyResult<Self> {
        match NativeSymbol::from_string(s) {
            Ok(sym) => Ok(Symbol { inner: sym }),
            Err(SymbolError::BadSymbolName) => {
                Err(pyo3::exceptions::PyValueError::new_err("Bad symbol name format"))
            }
            Err(SymbolError::InvalidSymbolCharacter) => {
                Err(pyo3::exceptions::PyValueError::new_err("Invalid symbol character"))
            }
            Err(SymbolError::InvalidPrecision) => {
                Err(pyo3::exceptions::PyValueError::new_err("Invalid symbol precision"))
            }
        }
    }

    #[getter]
    fn code(&self) -> SymbolCode {
        SymbolCode { inner: self.inner.code() }
    }

    #[getter]
    fn precision(&self) -> usize {
        self.inner.precision()
    }

    #[getter]
    fn unit(&self) -> f64 {
        1.0 / (10u64.pow(self.precision() as u32) as f64)
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __int__(&self) -> u64 {
        self.inner.value()
    }

    fn __richcmp__(&self, other: PyRef<Symbol>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}