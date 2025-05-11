use std::str::FromStr;
use antelope::chain::asset::{SymbolCode as NativeSymbolCode};
use antelope::serializer::Packer;
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use crate::impl_packable_py;

#[pyclass]
#[derive(Debug, Clone)]
pub struct SymbolCode {
    pub inner: NativeSymbolCode,
}

impl_packable_py! {
    impl SymbolCode(NativeSymbolCode) {
        #[staticmethod]
        pub fn from_str(sym: &str) -> PyResult<Self> {
            Ok(SymbolCode { inner: NativeSymbolCode::from_str(sym)
                .map_err(|e| PyValueError::new_err(e.to_string()))? })
        }

        #[staticmethod]
        pub fn from_int(sym: u64) -> PyResult<Self> {
            Ok(SymbolCode { inner: NativeSymbolCode::try_from(sym)
                .map_err(|e| PyValueError::new_err(e.to_string()))? })
        }

        #[getter]
        pub fn value(&self) -> u64 {
            self.inner.into()
        }

        fn __str__(&self) -> String {
            self.inner.to_string()
        }

        fn __int___(&self) -> u64 {
            self.inner.into()
        }

        fn __richcmp__(&self, other: PyRef<SymbolCode>, op: CompareOp) -> PyResult<bool> {
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