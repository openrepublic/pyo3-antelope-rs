use crate::proxies::public_key::PublicKey;
use antelope::chain::key_type::KeyType;
use antelope::chain::private_key::PrivateKey as NativePrivateKey;
use antelope::serializer::{Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PrivateKey {
    pub inner: NativePrivateKey,
}

#[derive(FromPyObject)]
pub enum PrivKeyLike {
    Raw(Vec<u8>),
    Str(String),
    Cls(PrivateKey)
}

impl From<PrivateKey> for NativePrivateKey {
    fn from(value: PrivateKey) -> Self {
        value.inner
    }
}

impl From<NativePrivateKey> for PrivateKey {
    fn from(value: NativePrivateKey) -> Self {
        PrivateKey { inner: value }
    }
}

#[pymethods]
impl PrivateKey {
    #[staticmethod]
    pub fn from_bytes(raw: &[u8]) -> PyResult<Self> {
        let key_type =
            KeyType::try_from(raw[0]).map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(NativePrivateKey::from_bytes(raw[1..].to_vec(), key_type).into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativePrivateKey::from_str(s)
            .map(|k| k.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: PrivKeyLike) -> PyResult<PrivateKey> {
        match value {
            PrivKeyLike::Raw(data) => PrivateKey::from_bytes(&data),
            PrivKeyLike::Str(s) => PrivateKey::from_str_py(&s),
            PrivKeyLike::Cls(key) => Ok(key)
        }
    }

    #[staticmethod]
    pub fn random(key_type: u8) -> PyResult<Self> {
        let key_type = KeyType::try_from(key_type)
            .map_err(|e| PyValueError::new_err(format!("Invalid key type format {e}")))?;

        let inner = NativePrivateKey::random(key_type)
            .map_err(|e| PyValueError::new_err(format!("Invalid key format {e}")))?;

        Ok(PrivateKey { inner })
    }

    pub fn value(&self) -> &[u8] {
        self.inner.value.as_slice()
    }

    pub fn get_public(&self) -> PyResult<PublicKey> {
        Ok(PublicKey {
            inner: self.inner.to_public()
                .map_err(|e| PyValueError::new_err(e.to_string()))?
        })
    }

    pub fn sign_message(&self, msg: Vec<u8>) -> PyResult<Vec<u8>> {
        let mut encoder = Encoder::new(0);
        let sig = self.inner.sign_message(&msg)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        sig.pack(&mut encoder);
        Ok(encoder.get_bytes().to_vec())
    }

    #[getter]
    pub fn raw(&self) -> &[u8] {
        &self.inner.value
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __hash__(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.inner.key_type.to_index().hash(&mut h);
        self.inner.value.hash(&mut h);
        h.finish()
    }

    fn __richcmp__(&self, other: &PrivateKey, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}
