use std::fmt::Display;

use antelope::chain::time::{BlockTimestamp, TimePoint, TimePointSec};
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};

use std::str::FromStr;

#[pyclass(frozen, name = "TimePoint")]
#[derive(Debug, Clone)]
pub struct PyTimePoint {
    pub inner: TimePoint,
}

#[derive(FromPyObject)]
pub enum TimePointLike {
    Raw(Vec<u8>),
    Int(u64),
    Str(String),
    Cls(PyTimePoint),
}

impl From<PyTimePoint> for TimePoint {
    fn from(value: PyTimePoint) -> Self {
        value.inner
    }
}

impl From<TimePoint> for PyTimePoint {
    fn from(value: TimePoint) -> Self {
        PyTimePoint { inner: value }
    }
}

#[pymethods]
impl PyTimePoint {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: TimePoint = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    pub fn from_int(num: u64) -> Self {
        TimePoint::from(num).into()
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        TimePoint::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[classmethod]
    pub fn try_from<'py>(_cls: &Bound<'py, PyType>, value: TimePointLike) -> PyResult<PyTimePoint> {
        match value {
            TimePointLike::Raw(data) => PyTimePoint::from_bytes(&data),
            TimePointLike::Int(num) => Ok(PyTimePoint::from_int(num)),
            TimePointLike::Str(s) => PyTimePoint::from_str_py(&s),
            TimePointLike::Cls(sum) => Ok(sum),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> u64 {
        self.inner.elapsed
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<PyTimePoint>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyTimePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass(frozen, name = "TimePointSec")]
#[derive(Debug, Clone)]
pub struct PyTimePointSec {
    pub inner: TimePointSec,
}

#[derive(FromPyObject)]
pub enum TimePointSecLike {
    Raw(Vec<u8>),
    Int(u32),
    Str(String),
    Cls(PyTimePointSec),
}

impl From<PyTimePointSec> for TimePointSec {
    fn from(value: PyTimePointSec) -> Self {
        value.inner
    }
}

impl From<TimePointSec> for PyTimePointSec {
    fn from(value: TimePointSec) -> Self {
        PyTimePointSec { inner: value }
    }
}

#[pymethods]
impl PyTimePointSec {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: TimePointSec = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    pub fn from_int(num: u32) -> Self {
        TimePointSec::from(num).into()
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        TimePointSec::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[classmethod]
    pub fn try_from<'py>(
        _cls: &Bound<'py, PyType>,
        value: TimePointSecLike,
    ) -> PyResult<PyTimePointSec> {
        match value {
            TimePointSecLike::Raw(data) => PyTimePointSec::from_bytes(&data),
            TimePointSecLike::Int(num) => Ok(PyTimePointSec::from_int(num)),
            TimePointSecLike::Str(s) => PyTimePointSec::from_str_py(&s),
            TimePointSecLike::Cls(sum) => Ok(sum),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> u32 {
        self.inner.seconds
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<PyTimePointSec>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyTimePointSec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass(frozen, name = "BlockTimestamp")]
#[derive(Debug, Clone)]
pub struct PyBlockTimestamp {
    pub inner: BlockTimestamp,
}

#[derive(FromPyObject)]
pub enum BlockTimestampLike {
    Raw(Vec<u8>),
    Int(u32),
    Str(String),
    Cls(PyBlockTimestamp),
}

impl From<PyBlockTimestamp> for BlockTimestamp {
    fn from(value: PyBlockTimestamp) -> Self {
        value.inner
    }
}

impl From<BlockTimestamp> for PyBlockTimestamp {
    fn from(value: BlockTimestamp) -> Self {
        PyBlockTimestamp { inner: value }
    }
}

#[pymethods]
impl PyBlockTimestamp {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: BlockTimestamp = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    pub fn from_int(num: u32) -> Self {
        BlockTimestamp::from(num).into()
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        BlockTimestamp::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[classmethod]
    pub fn try_from<'py>(
        _cls: &Bound<'py, PyType>,
        value: BlockTimestampLike,
    ) -> PyResult<PyBlockTimestamp> {
        match value {
            BlockTimestampLike::Raw(data) => PyBlockTimestamp::from_bytes(&data),
            BlockTimestampLike::Int(num) => Ok(PyBlockTimestamp::from_int(num)),
            BlockTimestampLike::Str(s) => PyBlockTimestamp::from_str_py(&s),
            BlockTimestampLike::Cls(sum) => Ok(sum),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> u32 {
        self.inner.slot
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<PyBlockTimestamp>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyBlockTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
