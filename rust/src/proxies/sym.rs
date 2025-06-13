use crate::proxies::sym_code::PySymbolCode;
use antelope::chain::asset::Symbol;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};
use std::fmt::Display;
use std::str::FromStr;

#[pyclass(frozen, name = "Symbol")]
#[derive(Debug, Clone)]
pub struct PySymbol {
    pub inner: Symbol,
}

#[derive(Debug, Clone, FromPyObject)]
pub enum SymLike {
    Raw(Vec<u8>),
    Str(String),
    Int(u64),
    Cls(PySymbol),
}

impl From<PySymbol> for Symbol {
    fn from(value: PySymbol) -> Self {
        value.inner
    }
}

impl From<Symbol> for PySymbol {
    fn from(value: Symbol) -> Self {
        PySymbol { inner: value }
    }
}

#[pymethods]
impl PySymbol {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: Symbol = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Symbol::from_str(s)
            .map(|s| s.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn from_int(sym: u64) -> PyResult<Self> {
        Ok(Symbol::from(sym).into())
    }

    #[classmethod]
    pub fn try_from<'py>(_cls: &Bound<'py, PyType>, value: SymLike) -> PyResult<PySymbol> {
        match value {
            SymLike::Raw(raw) => PySymbol::from_bytes(&raw),
            SymLike::Str(s) => PySymbol::from_str_py(&s),
            SymLike::Int(sym) => PySymbol::from_int(sym),
            SymLike::Cls(sym) => Ok(sym),
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
    pub fn code(&self) -> PySymbolCode {
        PySymbolCode {
            inner: self.inner.code(),
        }
    }

    #[getter]
    pub fn precision(&self) -> u8 {
        self.inner.precision()
    }

    #[getter]
    fn unit(&self) -> f64 {
        1.0 / (10u64.pow(self.precision() as u32) as f64)
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

    fn __int__(&self) -> u64 {
        self.inner.value()
    }

    fn __richcmp__(&self, other: PyRef<PySymbol>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PySymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
