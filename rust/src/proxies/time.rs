use std::fmt::Display;

use antelope::chain::time::{
    TimePoint, TimePointSec, BlockTimestamp
};
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use std::str::FromStr;

#[pyclass(frozen, name = "TimePoint")]
#[derive(Debug, Clone)]
pub struct PyTimePoint {
    pub inner: TimePoint,
}

#[derive(FromPyObject)]
pub enum TimePointLike {
    Raw([u8; 8]),
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
    pub fn from_bytes(buffer: [u8; 8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(&buffer);
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

    #[staticmethod]
    pub fn try_from(value: TimePointLike) -> PyResult<PyTimePoint> {
        match value {
            TimePointLike::Raw(data) => PyTimePoint::from_bytes(data),
            TimePointLike::Int(num) => Ok(PyTimePoint::from_int(num)),
            TimePointLike::Str(s) => PyTimePoint::from_str_py(&s),
            TimePointLike::Cls(sum) => Ok(sum),
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
