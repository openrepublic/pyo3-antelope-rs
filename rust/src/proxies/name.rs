use crate::impl_packable_py;
use antelope::chain::name::Name as NativeName;
use antelope::serializer::Packer;
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[pyclass]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Name {
    pub inner: NativeName,
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.inner.value())
    }
}

impl_packable_py! {
    impl Name(NativeName) {
        #[staticmethod]
        fn from_int(value: u64) -> PyResult<Self> {
            // If you'd like to mirror the original assertion, handle it as an error:
            let name = NativeName::try_from(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(Name { inner: name })
        }

        #[staticmethod]
        fn from_str(s: &str) -> PyResult<Self> {
            Ok(Name{
                inner: NativeName::from_str(s)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?
            })
        }

        pub fn value(&self) -> u64 {
            self.inner.value()
        }

        fn __str__(&self) -> PyResult<String> {
            self.inner.as_str()
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }

        fn __hash__(&self) -> u64 {
            self.inner.value()
        }

        fn __int__(&self) -> u64 {
            self.inner.value()
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
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.to_string())
    }
}
