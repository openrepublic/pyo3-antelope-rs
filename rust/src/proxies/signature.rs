use antelope::chain::signature::Signature as NativeSig;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::str::FromStr;

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Signature {
    pub inner: NativeSig,
}

#[derive(FromPyObject)]
pub enum SigLike {
    Raw(Vec<u8>),
    Str(String),
    Cls(Signature),
}

impl From<Signature> for NativeSig {
    fn from(value: Signature) -> Self {
        value.inner
    }
}

impl From<NativeSig> for Signature {
    fn from(value: NativeSig) -> Self {
        Signature { inner: value }
    }
}

#[pymethods]
impl Signature {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: NativeSig = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativeSig::from_str(s)
            .map(|s| s.into())
            .map_err(PyValueError::new_err)
    }

    #[staticmethod]
    pub fn try_from(value: SigLike) -> PyResult<Signature> {
        match value {
            SigLike::Raw(data) => Signature::from_bytes(&data),
            SigLike::Str(s) => Signature::from_str_py(&s),
            SigLike::Cls(key) => Ok(key),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
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
