use crate::proxies::private_key::PyPrivateKey;
use antelope::chain::public_key::PublicKey;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;

#[pyclass(frozen, name = "PublicKey")]
#[derive(Debug, Clone)]
pub struct PyPublicKey {
    pub inner: PublicKey,
}

#[derive(FromPyObject)]
pub enum PubKeyLike {
    Raw(Vec<u8>),
    Str(String),
    Cls(PyPublicKey),
}

impl From<PyPublicKey> for PublicKey {
    fn from(value: PyPublicKey) -> Self {
        value.inner
    }
}

impl From<PublicKey> for PyPublicKey {
    fn from(value: PublicKey) -> Self {
        PyPublicKey { inner: value }
    }
}

#[pymethods]
impl PyPublicKey {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: PublicKey = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        PublicKey::from_str(s)
            .map(|k| k.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn from_priv(p: &PyPrivateKey) -> PyResult<Self> {
        p.inner.to_public().map(|k| k.into()).map_err(|e| {
            PyValueError::new_err(format!(
                "Error while extracting public key from private: {e}"
            ))
        })
    }

    #[staticmethod]
    pub fn try_from(value: PubKeyLike) -> PyResult<PyPublicKey> {
        match value {
            PubKeyLike::Raw(data) => PyPublicKey::from_bytes(&data),
            PubKeyLike::Str(s) => PyPublicKey::from_str_py(&s),
            PubKeyLike::Cls(key) => Ok(key),
        }
    }

    #[getter]
    pub fn raw(&self) -> &[u8] {
        &self.inner.value
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __hash__(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.inner.key_type.to_index().hash(&mut h);
        self.inner.value.hash(&mut h);
        h.finish()
    }

    fn __richcmp__(&self, other: &PyPublicKey, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}
