use pyo3::prelude::*;
use antelope::chain::checksum::{
    Checksum160 as NativeSum160,
    Checksum256 as NativeSum256,
    Checksum512 as NativeSum512,
};
use antelope::serializer::Packer;
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use crate::impl_packable_py;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Checksum160 {
    pub inner: NativeSum160,
}

impl_packable_py! {
    impl Checksum160(NativeSum160) {
        #[staticmethod]
        fn from_str(s: &str) -> PyResult<Self> {
            Ok(Checksum160 {
                inner: NativeSum160::from_hex(s)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?
            })
        }

        fn __str__(&self) -> String {
            self.inner.as_string()
        }

        fn __richcmp__(&self, other: PyRef<Checksum160>, op: CompareOp) -> PyResult<bool> {
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

#[pyclass]
#[derive(Debug, Clone)]
pub struct Checksum256 {
    pub inner: NativeSum256,
}

impl_packable_py! {
    impl Checksum256(NativeSum256) {
        #[staticmethod]
        fn from_str(s: &str) -> PyResult<Self> {
            Ok(Checksum256 {
                inner: NativeSum256::from_hex(s)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?
            })
        }

        fn __str__(&self) -> String {
            self.inner.as_string()
        }

        fn __richcmp__(&self, other: PyRef<Checksum256>, op: CompareOp) -> PyResult<bool> {
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

#[pyclass]
#[derive(Debug, Clone)]
pub struct Checksum512 {
    pub inner: NativeSum512,
}

impl_packable_py! {
    impl Checksum512(NativeSum512) {
        #[staticmethod]
        fn from_str(s: &str) -> PyResult<Self> {
            Ok(Checksum512 {
                inner: NativeSum512::from_hex(s)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?
            })
        }

        fn __str__(&self) -> String {
            self.inner.as_string()
        }

        fn __richcmp__(&self, other: PyRef<Checksum512>, op: CompareOp) -> PyResult<bool> {
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
