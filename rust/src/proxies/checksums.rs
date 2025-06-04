use std::fmt::Display;
use std::str::FromStr;

use antelope::chain::checksum::{
    Checksum160, Checksum256, Checksum512,
};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(frozen, name = "Checksum160")]
#[derive(Debug, Clone)]
pub struct PyChecksum160 {
    pub inner: Checksum160,
}

#[derive(FromPyObject)]
pub enum Sum160Like {
    Raw([u8; 20]),
    Str(String),
    Cls(PyChecksum160),
}

impl From<PyChecksum160> for Checksum160 {
    fn from(value: PyChecksum160) -> Self {
        value.inner
    }
}

impl From<Checksum160> for PyChecksum160 {
    fn from(value: Checksum160) -> Self {
        PyChecksum160 { inner: value }
    }
}

#[pymethods]
impl PyChecksum160 {
    #[staticmethod]
    pub fn from_bytes(data: [u8; 20]) -> PyResult<Self> {
        Ok(Checksum160 { data }.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Checksum160::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: Sum160Like) -> PyResult<PyChecksum160> {
        match value {
            Sum160Like::Raw(data) => PyChecksum160::from_bytes(data),
            Sum160Like::Str(s) => PyChecksum160::from_str_py(&s),
            Sum160Like::Cls(sum) => Ok(sum),
        }
    }

    #[getter]
    pub fn raw(&self) -> &[u8; 20] {
        &self.inner.data
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<PyChecksum160>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyChecksum160 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass(frozen, name = "Checksum256")]
#[derive(Debug, Clone)]
pub struct PyChecksum256 {
    pub inner: Checksum256,
}

#[derive(FromPyObject)]
pub enum Sum256Like {
    Raw([u8; 32]),
    Str(String),
    Cls(PyChecksum256),
}

impl From<PyChecksum256> for Checksum256 {
    fn from(value: PyChecksum256) -> Self {
        value.inner
    }
}

impl From<Checksum256> for PyChecksum256 {
    fn from(value: Checksum256) -> Self {
        PyChecksum256 { inner: value }
    }
}

#[pymethods]
impl PyChecksum256 {
    #[staticmethod]
    pub fn from_bytes(data: [u8; 32]) -> PyResult<Self> {
        Ok(Checksum256 { data }.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Checksum256::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: Sum256Like) -> PyResult<PyChecksum256> {
        match value {
            Sum256Like::Raw(data) => PyChecksum256::from_bytes(data),
            Sum256Like::Str(s) => PyChecksum256::from_str_py(&s),
            Sum256Like::Cls(sum) => Ok(sum),
        }
    }

    #[getter]
    pub fn raw(&self) -> &[u8; 32] {
        &self.inner.data
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<PyChecksum256>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyChecksum256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass(frozen, name = "Checksum512")]
#[derive(Debug, Clone)]
pub struct PyChecksum512 {
    pub inner: Checksum512,
}

#[derive(FromPyObject)]
pub enum Sum512Like {
    Raw([u8; 64]),
    Str(String),
    Cls(PyChecksum512),
}

impl From<PyChecksum512> for Checksum512 {
    fn from(value: PyChecksum512) -> Self {
        value.inner
    }
}

impl From<Checksum512> for PyChecksum512 {
    fn from(value: Checksum512) -> Self {
        PyChecksum512 { inner: value }
    }
}

#[pymethods]
impl PyChecksum512 {
    #[staticmethod]
    pub fn from_bytes(data: [u8; 64]) -> PyResult<Self> {
        Ok(Checksum512 { data }.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Checksum512::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: Sum512Like) -> PyResult<PyChecksum512> {
        match value {
            Sum512Like::Raw(data) => PyChecksum512::from_bytes(data),
            Sum512Like::Str(s) => PyChecksum512::from_str_py(&s),
            Sum512Like::Cls(sum) => Ok(sum),
        }
    }

    #[getter]
    pub fn raw(&self) -> &[u8; 64] {
        &self.inner.data
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<PyChecksum512>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyChecksum512 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
