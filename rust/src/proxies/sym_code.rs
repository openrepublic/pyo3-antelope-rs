use antelope::chain::asset::SymbolCode;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};
use std::fmt::Display;
use std::str::FromStr;

#[pyclass(frozen, name = "SymbolCode")]
#[derive(Debug, Clone)]
pub struct PySymbolCode {
    pub inner: SymbolCode,
}

#[derive(Debug, Clone, FromPyObject)]
pub enum SymCodeLike {
    Raw(Vec<u8>),
    Str(String),
    Int(u64),
    Cls(PySymbolCode),
}

impl From<PySymbolCode> for SymbolCode {
    fn from(value: PySymbolCode) -> Self {
        value.inner
    }
}

impl From<SymbolCode> for PySymbolCode {
    fn from(value: SymbolCode) -> Self {
        PySymbolCode { inner: value }
    }
}

#[pymethods]
impl PySymbolCode {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: SymbolCode = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(sym: &str) -> PyResult<Self> {
        Ok(PySymbolCode {
            inner: SymbolCode::from_str(sym).map_err(|e| PyValueError::new_err(e.to_string()))?,
        })
    }

    #[staticmethod]
    pub fn from_int(sym: u64) -> PyResult<Self> {
        Ok(PySymbolCode {
            inner: SymbolCode::try_from(sym).map_err(|e| PyValueError::new_err(e.to_string()))?,
        })
    }

    #[classmethod]
    pub fn try_from<'py>(_cls: &Bound<'py, PyType>, value: SymCodeLike) -> PyResult<PySymbolCode> {
        match value {
            SymCodeLike::Raw(raw) => PySymbolCode::from_bytes(&raw),
            SymCodeLike::Str(s) => PySymbolCode::from_str_py(&s),
            SymCodeLike::Int(sym) => PySymbolCode::from_int(sym),
            SymCodeLike::Cls(sym) => Ok(sym),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> String {
        self.inner.to_string()
    }

    #[getter]
    pub fn value(&self) -> u64 {
        self.inner.value()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    #[getter]
    pub fn encode_length(&self) -> usize {
        8
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __int___(&self) -> u64 {
        self.inner.value()
    }

    fn __richcmp__(&self, other: PyRef<PySymbolCode>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PySymbolCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
