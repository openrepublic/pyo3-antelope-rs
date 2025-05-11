use std::fmt::Display;
use std::hash::{Hash, Hasher};
use antelope::serializer::Packer;
use antelope::chain::name::{Name as NativeName};
use packvm::Value;
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use crate::impl_packable_py;
use crate::types::{AntelopeTypes, AntelopeValue};

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
            let name = NativeName::from(value);
            Ok(Name { inner: name })
        }

        #[staticmethod]
        fn from_str(s: &str) -> PyResult<Self> {
            Ok(Name{
                inner: NativeName::try_from(s)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?
            })
        }

        pub fn value(&self) -> u64 {
            self.inner.into()
        }

        fn __str__(&self) -> PyResult<String> {
            self.inner.as_string()
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

impl TryFrom<&Value> for Name {
    type Error = PyErr;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Name::from_str(&s),
            Value::Int(int) => {
                let num = int.as_u64()
                    .ok_or(PyValueError::new_err(format!("Cant cast {:?} to u64", int)))?;

                Name::from_int(num)
            }
            _ => Err(PyTypeError::new_err(format!("Cant convert {:?} to Name", value))),
        }
    }
}

impl TryFrom<&AntelopeValue> for Name {
    type Error = PyErr;

    fn try_from(value: &AntelopeValue) -> Result<Self, Self::Error> {
        match value {
            AntelopeValue::Generic(val) => val.try_into(),
            AntelopeValue::Antelope(wrapper) => {
                match wrapper {
                    AntelopeTypes::Name(name) => Ok(name.clone()),
                    _ => Err(PyTypeError::new_err(format!("Can not convert {:?} to Name", wrapper)))
                }
            }
            _ => Err(PyTypeError::new_err(format!("Can not convert {:?} to Name", value)))
        }
    }
}