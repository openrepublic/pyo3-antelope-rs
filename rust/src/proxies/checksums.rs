use std::fmt::Display;

use antelope::chain::checksum::{
    Checksum160 as NativeSum160, Checksum256 as NativeSum256, Checksum512 as NativeSum512,
};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Checksum160 {
    pub inner: NativeSum160,
}

#[derive(FromPyObject)]
pub enum Sum160Like {
    Raw([u8; 20]),
    Str(String),
    Cls(Checksum160)
}

impl From<Checksum160> for NativeSum160 {
    fn from(value: Checksum160) -> Self {
        value.inner
    }
}

impl From<NativeSum160> for Checksum160 {
    fn from(value: NativeSum160) -> Self {
        Checksum160 { inner: value }
    }
}

#[pymethods]
impl Checksum160 {
    #[staticmethod]
    pub fn from_bytes(
        data: [u8; 20],
    ) -> PyResult<Self> {
        Ok(NativeSum160 { data }.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativeSum160::from_hex(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: Sum160Like) -> PyResult<Checksum160> {
        match value {
            Sum160Like::Raw(data) => Checksum160::from_bytes(data),
            Sum160Like::Str(s) => Checksum160::from_str_py(&s),
            Sum160Like::Cls(sum) => Ok(sum)
        }
    }

    #[getter]
    pub fn raw(&self) -> [u8; 20] {
        self.inner.data
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __richcmp__(&self, other: PyRef<Checksum160>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for Checksum160 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Checksum256 {
    pub inner: NativeSum256,
}

#[derive(FromPyObject)]
pub enum Sum256Like {
    Raw([u8; 32]),
    Str(String),
    Cls(Checksum256)
}

impl From<Checksum256> for NativeSum256 {
    fn from(value: Checksum256) -> Self {
        value.inner
    }
}

impl From<NativeSum256> for Checksum256 {
    fn from(value: NativeSum256) -> Self {
        Checksum256 { inner: value }
    }
}

#[pymethods]
impl Checksum256 {
    #[staticmethod]
    pub fn from_bytes(
        data: [u8; 32],
    ) -> PyResult<Self> {
        Ok(NativeSum256 { data }.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativeSum256::from_hex(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: Sum256Like) -> PyResult<Checksum256> {
        match value {
            Sum256Like::Raw(data) => Checksum256::from_bytes(data),
            Sum256Like::Str(s) => Checksum256::from_str_py(&s),
            Sum256Like::Cls(sum) => Ok(sum)
        }
    }

    #[getter]
    pub fn raw(&self) -> [u8; 32] {
        self.inner.data
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __richcmp__(&self, other: PyRef<Checksum256>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for Checksum256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Checksum512 {
    pub inner: NativeSum512,
}

#[derive(FromPyObject)]
pub enum Sum512Like {
    Raw([u8; 64]),
    Str(String),
    Cls(Checksum512)
}

impl From<Checksum512> for NativeSum512 {
    fn from(value: Checksum512) -> Self {
        value.inner
    }
}

impl From<NativeSum512> for Checksum512 {
    fn from(value: NativeSum512) -> Self {
        Checksum512 { inner: value }
    }
}

#[pymethods]
impl Checksum512 {
    #[staticmethod]
    pub fn from_bytes(
        data: [u8; 64],
    ) -> PyResult<Self> {
        Ok(NativeSum512 { data }.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativeSum512::from_hex(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: Sum512Like) -> PyResult<Checksum512> {
        match value {
            Sum512Like::Raw(data) => Checksum512::from_bytes(data),
            Sum512Like::Str(s) => Checksum512::from_str_py(&s),
            Sum512Like::Cls(sum) => Ok(sum)
        }
    }

    #[getter]
    pub fn raw(&self) -> [u8; 64] {
        self.inner.data
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __richcmp__(&self, other: PyRef<Checksum512>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for Checksum512 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
