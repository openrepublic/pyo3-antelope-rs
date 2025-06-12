use std::fmt::Display;
use std::ops::{Add, Div, Mul, Sub};

use antelope::chain::varint::{VarInt32, VarUint32};
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};

use std::str::FromStr;

#[pyclass(frozen, name = "VarUInt32")]
#[derive(Debug, Copy, Clone)]
pub struct PyVarUInt32 {
    pub inner: VarUint32,
}

#[derive(FromPyObject)]
pub enum VarUInt32Like {
    Raw(Vec<u8>),
    Str(String),
    Int(u32),
    Cls(PyVarUInt32),
}

impl From<PyVarUInt32> for VarUint32 {
    fn from(value: PyVarUInt32) -> Self {
        value.inner
    }
}

impl From<VarUint32> for PyVarUInt32 {
    fn from(value: VarUint32) -> Self {
        PyVarUInt32 { inner: value }
    }
}

#[pymethods]
impl PyVarUInt32 {
    #[staticmethod]
    pub fn from_int(n: u32) -> Self {
        VarUint32::from(n).into()
    }

    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: VarUint32 = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        VarUint32::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[classmethod]
    pub fn try_from<'py>(_cls: &Bound<'py, PyType>, value: VarUInt32Like) -> PyResult<PyVarUInt32> {
        match value {
            VarUInt32Like::Raw(data) => PyVarUInt32::from_bytes(&data),
            VarUInt32Like::Str(s) => PyVarUInt32::from_str_py(&s),
            VarUInt32Like::Int(num) => Ok(PyVarUInt32::from_int(num)),
            VarUInt32Like::Cls(sum) => Ok(sum),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> u32 {
        self.inner.n
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    #[getter]
    fn encode_length(&self) -> usize {
        self.inner.size()
    }

    fn __int__(&self) -> u32 {
        self.inner.n
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __add__(&self, other: &PyVarUInt32) -> PyVarUInt32 {
        *self + *other
    }

    fn __sub__(&self, other: &PyVarUInt32) -> PyVarUInt32 {
        *self - *other
    }

    fn __mul__(&self, other: &PyVarUInt32) -> PyVarUInt32 {
        *self * *other
    }

    fn __div__(&self, other: &PyVarUInt32) -> PyVarUInt32 {
        *self / *other
    }

    fn __richcmp__(&self, other: PyRef<PyVarUInt32>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyVarUInt32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Add for PyVarUInt32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        PyVarUInt32::from(self.inner + rhs.inner)
    }
}

impl Sub for PyVarUInt32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        PyVarUInt32::from(self.inner - rhs.inner)
    }
}

impl Mul for PyVarUInt32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        PyVarUInt32::from(self.inner * rhs.inner)
    }
}

impl Div for PyVarUInt32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        PyVarUInt32::from(self.inner / rhs.inner)
    }
}

#[pyclass(frozen, name = "VarInt32")]
#[derive(Debug, Copy, Clone)]
pub struct PyVarInt32 {
    pub inner: VarInt32,
}

#[derive(FromPyObject)]
pub enum VarInt32Like {
    Raw(Vec<u8>),
    Str(String),
    Int(i32),
    Cls(PyVarInt32),
}

impl From<PyVarInt32> for VarInt32 {
    fn from(value: PyVarInt32) -> Self {
        value.inner
    }
}

impl From<VarInt32> for PyVarInt32 {
    fn from(value: VarInt32) -> Self {
        PyVarInt32 { inner: value }
    }
}

#[pymethods]
impl PyVarInt32 {
    #[staticmethod]
    pub fn from_int(n: i32) -> Self {
        VarInt32::from(n).into()
    }

    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: VarInt32 = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        VarInt32::from_str(s)
            .map(|sum| sum.into())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[classmethod]
    pub fn try_from<'py>(_cls: &Bound<'py, PyType>, value: VarInt32Like) -> PyResult<PyVarInt32> {
        match value {
            VarInt32Like::Raw(data) => PyVarInt32::from_bytes(&data),
            VarInt32Like::Str(s) => PyVarInt32::from_str_py(&s),
            VarInt32Like::Int(num) => Ok(PyVarInt32::from_int(num)),
            VarInt32Like::Cls(sum) => Ok(sum),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> i32 {
        self.inner.n
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __int__(&self) -> i32 {
        self.inner.n
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __add__(&self, other: &PyVarInt32) -> PyVarInt32 {
        *self + *other
    }

    fn __sub__(&self, other: &PyVarInt32) -> PyVarInt32 {
        *self - *other
    }

    fn __mul__(&self, other: &PyVarInt32) -> PyVarInt32 {
        *self * *other
    }

    fn __div__(&self, other: &PyVarInt32) -> PyVarInt32 {
        *self / *other
    }

    fn __richcmp__(&self, other: PyRef<PyVarInt32>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyVarInt32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Add for PyVarInt32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        PyVarInt32::from(self.inner + rhs.inner)
    }
}

impl Sub for PyVarInt32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        PyVarInt32::from(self.inner - rhs.inner)
    }
}

impl Mul for PyVarInt32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        PyVarInt32::from(self.inner * rhs.inner)
    }
}

impl Div for PyVarInt32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        PyVarInt32::from(self.inner / rhs.inner)
    }
}
