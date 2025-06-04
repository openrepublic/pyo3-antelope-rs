use antelope::chain::name::Name as NativeName;
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[pyclass]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Name {
    pub inner: NativeName,
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.inner.value())
    }
}

#[derive(FromPyObject)]
pub enum NameLike {
    Num(u64),
    Raw([u8; 8]),
    Str(String),
    Cls(Name),
}

impl From<Name> for NativeName {
    fn from(value: Name) -> Self {
        value.inner
    }
}

impl From<NativeName> for Name {
    fn from(value: NativeName) -> Self {
        Name { inner: value }
    }
}

#[pymethods]
impl Name {
    #[staticmethod]
    pub fn from_int(value: u64) -> PyResult<Self> {
        Ok(NativeName::from(value).into())
    }

    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: NativeName = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativeName::from_str(s)
            .map(|n| n.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn try_from(value: NameLike) -> PyResult<Name> {
        match value {
            NameLike::Num(n) => Name::from_int(n),
            NameLike::Raw(raw) => Name::from_bytes(&raw),
            NameLike::Str(n_str) => Name::from_str_py(&n_str),
            NameLike::Cls(n) => Ok(n.clone()),
        }
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

    fn __richcmp__(&self, other: &Name, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
