use std::str::FromStr;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use antelope::chain::asset::{Asset as NativeAsset, Symbol as NativeSymbol};
use pyo3::exceptions::PyValueError;
use rust_decimal::Decimal;
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

    #[staticmethod]
    fn from_decimal(d: Decimal, precision: u8, sym: &str) -> PyResult<Self> {
        let d_str = d.to_string();
        let dot_idx = d_str.find('.')
            .unwrap_or(Err(PyValueError::new_err("Could not find decimal point"))?);

        let num_str = d_str[..dot_idx + 1 + precision as usize].to_string();
        Ok(Asset::from_str(&format!("{} {}", num_str, sym))?)
    }

    fn to_decimal(&self) -> Decimal {
        let mut str_amount = format!("{:0>width$}", self.amount(), width = self.symbol().precision() + 1);

        if self.symbol().precision() > 0 {
            let len = str_amount.len();
            str_amount.insert(len - self.symbol().precision() as usize, '.');
        }

        Decimal::from_str(&str_amount).unwrap_or(Decimal::ZERO)
    }

    /// Return the i64 amount
    #[getter]
    pub fn amount(&self) -> i64 {
        self.inner.amount()
    }

    #[getter]
    pub fn symbol(&self) -> Symbol {
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