use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use antelope::chain::asset::{Asset as NativeAsset, Symbol as NativeSymbol};
use pyo3::exceptions::PyValueError;
use crate::proxies::sym::Symbol;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Asset {
    pub inner: NativeAsset,
}

#[pymethods]
impl Asset {
    #[new]
    fn new(amount: i64, sym_val: Py<PyAny>) -> PyResult<Self> {
        let sym = Python::with_gil(|py| {
            if let Ok(sym_str) = sym_val.extract::<String>(py) {
                return Ok(NativeSymbol::from_string(&sym_str)
                    .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?);
            } else if let Ok(sym) = sym_val.extract::<Symbol>(py) {
                return Ok(sym.inner);
            }
            Err(PyErr::new::<PyValueError, _>("Could not convert provided to symbol"))
        })?;
        let inner = NativeAsset::new(amount, sym);
        Ok(Asset { inner })
    }
    #[staticmethod]
    fn from_str(s: &str) -> PyResult<Self> {
        match NativeAsset::from_string(s) {
            Ok(asset) => Ok(Asset { inner: asset }),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid asset string: {}",
                e
            ))),
        }
    }

    #[staticmethod]
    fn from_ints(amount: i64, precision: u8, sym: &str) -> PyResult<Self> {
        Ok(Asset {
            inner: NativeAsset::new(amount, NativeSymbol::new(sym, precision)),
        })
    }

    /// Return the i64 amount
    #[getter]
    fn amount(&self) -> i64 {
        self.inner.amount()
    }

    #[getter]
    fn symbol(&self) -> Symbol {
        Symbol {
            inner: self.inner.symbol(),
        }
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __richcmp__(&self, other: PyRef<Asset>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }

    fn __add__(&self, other: &Asset) -> PyResult<Asset> {
        let result = self.inner + other.inner;
        Ok(Asset { inner: result })
    }

    fn __sub__(&self, other: &Asset) -> PyResult<Asset> {
        let result = self.inner - other.inner;
        Ok(Asset { inner: result })
    }
}