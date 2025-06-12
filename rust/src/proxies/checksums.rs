use std::fmt::Display;

use antelope::chain::checksum::{Checksum160, Checksum256, Checksum512};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};

use crate::utils::try_decode_string_bytes;

/// Generate Python‑facing checksum wrappers with PyO3.
///
/// Each tuple argument expands to a complete wrapper:
/// `(WrapperIdent, "PythonName", LikeEnumIdent, NativeIdent, BYTES)`
macro_rules! define_checksum_py {
    ($(($wrapper:ident, $py_name:literal, $like:ident, $native:ident, $len:expr)),+ $(,)?) => {
        $(
            #[pyclass(frozen, name = $py_name)]
            #[derive(Debug, Clone)]
            pub struct $wrapper {
                pub inner: $native,
            }

            #[derive(FromPyObject)]
            pub enum $like {
                Raw(Vec<u8>),
                Str(String),
                Cls($wrapper),
            }

            impl From<$wrapper> for $native {
                fn from(value: $wrapper) -> Self { value.inner }
            }

            impl From<$native> for $wrapper {
                fn from(value: $native) -> Self { Self { inner: value } }
            }

            #[pymethods]
            impl $wrapper {
                #[staticmethod]
                pub fn from_bytes(data: &[u8]) -> PyResult<Self> {
                    if data.len() < $len {
                        return Err(PyValueError::new_err(format!(
                            "Expected at least {0} bytes, got {1}", $len, data.len()
                        )));
                    }
                    Ok($native { data: data[..$len].try_into().expect("slice len verified") }.into())
                }

                #[staticmethod]
                #[pyo3(name = "from_str")]
                pub fn from_str_py(s: &str) -> PyResult<Self> {
                    let bytes = try_decode_string_bytes(s, Some($len))?;
                    Self::from_bytes(&bytes)
                }

                #[classmethod]
                pub fn try_from<'py>(_cls: &Bound<'py, PyType>, value: $like) -> PyResult<Self> {
                    match value {
                        $like::Raw(d) => Self::from_bytes(&d),
                        $like::Str(s) => Self::from_str_py(&s),
                        $like::Cls(c) => Ok(c),
                    }
                }

                #[classmethod]
                pub fn pretty_def_str<'py>(cls: &Bound<'py, PyType>) -> PyResult<Bound<'py, PyString>> {
                    cls.name()
                }

                pub fn to_builtins(&self) -> &[u8; $len] { &self.inner.data }
                pub fn encode(&self) -> &[u8; $len]     { &self.inner.data }

                fn __str__(&self) -> String { self.inner.to_string() }

                fn __richcmp__(&self, other: PyRef<$wrapper>, op: CompareOp) -> PyResult<bool> {
                    let rhs = &(*other).inner; // deref PyRef to access the wrapped struct
                    Ok(match op {
                        CompareOp::Eq => self.inner == *rhs,
                        CompareOp::Ne => self.inner != *rhs,
                        _ => return Err(pyo3::exceptions::PyNotImplementedError::new_err(
                            "Operation not implemented",
                        )),
                    })
                }
            }

            impl Display for $wrapper {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.inner)
                }
            }
        )+
    };
}

define_checksum_py!(
    (PyChecksum160, "Checksum160", Sum160Like, Checksum160, 20),
    (PyChecksum256, "Checksum256", Sum256Like, Checksum256, 32),
    (PyChecksum512, "Checksum512", Sum512Like, Checksum512, 64),
);
