use antelope::chain::asset::{SymbolCode as NativeSymbolCode, SymbolCodeError};
use antelope::serializer::Packer;
use pyo3::basic::CompareOp;
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
            match NativeSymbolCode::from_string(sym) {
                Ok(code) => Ok(SymbolCode { inner: code }),
                Err(SymbolCodeError::BadSymbolName) => {
                    Err(pyo3::exceptions::PyValueError::new_err("Bad symbol name"))
                }
                Err(SymbolCodeError::InvalidSymbolCharacter) => {
                    Err(pyo3::exceptions::PyValueError::new_err("Invalid symbol character"))
                }
            }
        }

        #[staticmethod]
        pub fn from_int(sym: u64) -> PyResult<Self> {
            Ok(SymbolCode { inner: NativeSymbolCode { value: sym }})
        }

        #[getter]
        pub fn value(&self) -> u64 {
            self.inner.value
        }

        fn __str__(&self) -> String {
            self.inner.as_string()
        }

        fn __int___(&self) -> u64 {
            self.inner.value
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