use antelope::chain::key_type::{KeyType, KeyTypeTrait};
use antelope::chain::signature::Signature as NativeSig;
use antelope::serializer::{Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Signature {
    pub inner: NativeSig,
}

#[pymethods]
impl Signature {
    #[staticmethod]
    fn from_str(s: &str) -> PyResult<Self> {
        Ok(Signature {
            inner: NativeSig::from_string(s).map_err(|e| PyValueError::new_err(e))?,
        })
    }

    #[staticmethod]
    fn from_bytes(raw: &[u8]) -> PyResult<Self> {
        let key_type =
            KeyType::from_index(raw[0]).map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(Signature {
            inner: NativeSig::from_bytes(raw.to_vec(), key_type), // .map_err(|e| PyValueError::new_err(e))?,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __richcmp__(&self, other: &Signature, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}
