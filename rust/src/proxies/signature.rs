use antelope::chain::signature::Signature;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};
use std::str::FromStr;

use super::TryFromError;

#[pyclass(frozen, name = "Signature")]
#[derive(Debug, Clone)]
pub struct PySignature {
    pub inner: Signature,
}

#[derive(FromPyObject)]
pub enum SigLike {
    Raw(Vec<u8>),
    Str(String),
    Cls(PySignature),
}

impl From<PySignature> for Signature {
    fn from(value: PySignature) -> Self {
        value.inner
    }
}

impl From<Signature> for PySignature {
    fn from(value: Signature) -> Self {
        PySignature { inner: value }
    }
}

#[pymethods]
impl PySignature {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: Signature = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| TryFromError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Signature::from_str(s)
            .map(|s| s.into())
            .map_err(TryFromError::new_err)
    }

    #[classmethod]
    pub fn try_from<'py>(_cls: &Bound<'py, PyType>, value: SigLike) -> PyResult<PySignature> {
        match value {
            SigLike::Raw(data) => PySignature::from_bytes(&data),
            SigLike::Str(s) => PySignature::from_str_py(&s),
            SigLike::Cls(key) => Ok(key),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> Vec<u8> {
        self.encode()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    #[getter]
    pub fn encode_length(&self) -> usize {
        self.inner.size()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: &PySignature, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}
