use std::str::FromStr;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use antelope::chain::asset::{Symbol as NativeSymbol};
use antelope::serializer::Packer;
use pyo3::exceptions::PyValueError;
use crate::impl_packable_py;
use crate::proxies::sym_code::SymbolCode;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Symbol {
    pub inner: NativeSymbol,
}

impl_packable_py! {
    impl Symbol(NativeSymbol) {
        #[staticmethod]
        pub fn from_str(s: &str) -> PyResult<Self> {
            Ok(Symbol { inner: NativeSymbol::from_str(s)
                .map_err(|e| PyValueError::new_err(e.to_string()))? })
        }

        #[staticmethod]
        pub fn from_int(sym: u64) -> PyResult<Self> {
            Ok(Symbol { inner: sym.into()})
        }

        #[getter]
        pub fn code(&self) -> SymbolCode {
            SymbolCode { inner: self.inner.code() }
        }

        #[getter]
        pub fn precision(&self) -> usize {
            self.inner.precision()
        }

        #[getter]
        fn unit(&self) -> f64 {
            1.0 / (10u64.pow(self.precision() as u32) as f64)
        }

        fn __str__(&self) -> String {
            self.inner.to_string()
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
}