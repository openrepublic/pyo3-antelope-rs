use std::fmt::Display;
use std::ops::{Add, Div, Mul, Sub};

use antelope::chain::float::Float128;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyFloat;

use std::str::FromStr;

#[pyclass(frozen, name = "Float128")]
#[derive(Debug, Copy, Clone)]
pub struct PyFloat128 {
    pub inner: Float128,
}

#[derive(FromPyObject)]
pub enum Float128Like<'py> {
    Raw([u8; 16]),
    Str(String),
    Float(Bound<'py, PyFloat>),
    Cls(PyFloat128),
}

impl From<PyFloat128> for Float128 {
    fn from(value: PyFloat128) -> Self {
        value.inner
    }
}

impl From<Float128> for PyFloat128 {
    fn from(value: Float128) -> Self {
        PyFloat128 { inner: value }
    }
}

#[pymethods]
impl PyFloat128 {
    #[staticmethod]
    pub fn from_bytes(buffer: [u8; 16]) -> PyResult<Self> {
        let mut decoder = Decoder::new(&buffer);
        let mut inner: Float128 = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Float128::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: Float128Like) -> PyResult<PyFloat128> {
        match value {
            Float128Like::Raw(data) => PyFloat128::from_bytes(data),
            Float128Like::Str(s) => PyFloat128::from_str_py(&s),
            Float128Like::Float(py_float) => PyFloat128::from_str_py(&py_float.to_string()),
            Float128Like::Cls(sum) => Ok(sum),
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

    fn __add__(&self, other: &PyFloat128) -> PyFloat128 {
        *self + *other
    }

    fn __sub__(&self, other: &PyFloat128) -> PyFloat128 {
        *self - *other
    }

    fn __mul__(&self, other: &PyFloat128) -> PyFloat128 {
        *self * *other
    }

    fn __div__(&self, other: &PyFloat128) -> PyFloat128 {
        *self / *other
    }

    fn __richcmp__(&self, other: PyRef<PyFloat128>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyFloat128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Add for PyFloat128 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        PyFloat128::from(self.inner + rhs.inner)
    }
}

impl Sub for PyFloat128 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        PyFloat128::from(self.inner - rhs.inner)
    }
}

impl Mul for PyFloat128 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        PyFloat128::from(self.inner * rhs.inner)
    }
}

impl Div for PyFloat128 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        PyFloat128::from(self.inner / rhs.inner)
    }
}
