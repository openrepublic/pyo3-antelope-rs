use crate::proxies::sym_code::SymbolCode;
use antelope::chain::asset::Symbol as NativeSymbol;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fmt::Display;
use std::str::FromStr;

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Symbol {
    pub inner: NativeSymbol,
}

#[derive(Debug, Clone, FromPyObject)]
pub enum SymLike {
    Raw([u8; 8]),
    Str(String),
    Int(u64),
    Cls(Symbol),
}

impl From<Symbol> for NativeSymbol {
    fn from(value: Symbol) -> Self {
        value.inner
    }
}

impl From<NativeSymbol> for Symbol {
    fn from(value: NativeSymbol) -> Self {
        Symbol { inner: value }
    }
}

#[pymethods]
impl Symbol {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: NativeSymbol = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativeSymbol::from_str(s)
            .map(|s| s.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn from_int(sym: u64) -> PyResult<Self> {
        Ok(NativeSymbol::from(sym).into())
    }

    #[staticmethod]
    pub fn try_from(value: SymLike) -> PyResult<Symbol> {
        match value {
            SymLike::Raw(raw) => Symbol::from_bytes(&raw),
            SymLike::Str(s) => Symbol::from_str_py(&s),
            SymLike::Int(sym) => Symbol::from_int(sym),
            SymLike::Cls(sym) => Ok(sym),
        }
    }

    #[getter]
    pub fn code(&self) -> SymbolCode {
        SymbolCode {
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

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __int__(&self) -> u64 {
        self.inner.value()
    }

    fn __richcmp__(&self, other: PyRef<Symbol>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
