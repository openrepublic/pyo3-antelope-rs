use antelope::chain::name::Name;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[pyclass(frozen, name = "Name")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PyName {
    pub inner: Name,
}

impl Hash for PyName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.inner.value())
    }
}

#[derive(FromPyObject)]
pub enum NameLike {
    Num(u64),
    Raw(Vec<u8>),
    Str(String),
    Cls(PyName),
}

impl From<PyName> for Name {
    fn from(value: PyName) -> Self {
        value.inner
    }
}

impl From<Name> for PyName {
    fn from(value: Name) -> Self {
        PyName { inner: value }
    }
}

#[pymethods]
impl PyName {
    #[staticmethod]
    pub fn from_int(value: u64) -> PyResult<Self> {
        Ok(Name::from(value).into())
    }

    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: Name = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Name::from_str(s)
            .map(|n| n.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[classmethod]
    pub fn try_from<'py>(
        _cls: &Bound<'py, PyType>,
        value: NameLike
    ) -> PyResult<PyName> {
        match value {
            NameLike::Num(n) => PyName::from_int(n),
            NameLike::Raw(raw) => PyName::from_bytes(&raw),
            NameLike::Str(n_str) => PyName::from_str_py(&n_str),
            NameLike::Cls(n) => Ok(n.clone()),
        }
    }

    #[classmethod]
    pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
        cls.name()
    }

    pub fn to_builtins(&self) -> u64 {
        self.inner.value()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    pub fn value(&self) -> u64 {
        self.inner.value()
    }

    fn __str__(&self) -> PyResult<String> {
        self.inner
            .as_str()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn __hash__(&self) -> u64 {
        self.inner.value()
    }

    fn __int__(&self) -> u64 {
        self.inner.value()
    }

    fn __richcmp__(&self, other: &PyName, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for PyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
