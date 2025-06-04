use crate::serializer::{decode::decode_abi_type, encode::encode_abi_type};
use antelope::chain::abi::{
    ABITypeResolver, AbiStruct, AbiTableView, AbiVariant, ShipABI as NativeShipABI,
    ABI as NativeABI,
};
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::ser::Serialize;
use serde_json::Serializer;

fn abi_struct_as_dict<'py>(py: Python<'py>, s: &AbiStruct) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("name", s.name.clone())?;
    d.set_item("base", s.base.clone())?;
    let mut fields = Vec::with_capacity(s.fields.len());
    for fmeta in s.fields.iter() {
        let f = PyDict::new(py);
        f.set_item("name", fmeta.name.clone())?;
        f.set_item("type", fmeta.r#type.clone())?;
        fields.push(f);
    }
    d.set_item("fields", fields)?;
    Ok(d)
}

fn abi_variant_as_dict<'py>(py: Python<'py>, v: &AbiVariant) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("name", v.name.clone())?;
    d.set_item("types", v.types.clone())?;
    Ok(d)
}

macro_rules! define_pyabi {
    ($wrapper:ident, $inner:path) => {
        #[pyclass(frozen)]
        #[derive(Debug, Clone)]
        pub struct $wrapper {
            pub inner: $inner,
        }

        #[pymethods]
        impl $wrapper {
            #[staticmethod]
            pub fn from_bytes(buf: &[u8]) -> PyResult<Self> {
                let mut decoder = Decoder::new(buf);
                let mut inner = <$inner>::default();
                decoder
                    .unpack(&mut inner)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(Self { inner })
            }

            #[staticmethod]
            #[pyo3(name = "from_str")]
            pub fn from_str_py(s: &str) -> PyResult<Self> {
                let inner =
                    <$inner>::from_string(s).map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(Self { inner })
            }

            #[getter]
            pub fn version(&self) -> &String {
                &self.inner.version
            }

            #[getter]
            pub fn _types<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
                let mut ret = Vec::new();
                for t in self.inner.types.iter() {
                    let d = PyDict::new(py);
                    d.set_item("new_type_name", t.new_type_name.clone())?;
                    d.set_item("type", t.r#type.clone())?;
                    ret.push(d);
                }
                Ok(ret)
            }

            #[getter]
            pub fn _structs<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
                let mut ret = Vec::new();
                for s in self.inner.structs.iter() {
                    ret.push(abi_struct_as_dict(py, s)?);
                }
                Ok(ret)
            }

            #[getter]
            pub fn _variants<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
                let mut ret = Vec::new();
                for v in self.inner.variants.iter() {
                    ret.push(abi_variant_as_dict(py, v)?);
                }
                Ok(ret)
            }

            #[getter]
            pub fn _actions<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
                let mut ret = Vec::new();
                for a in self.inner.actions.iter() {
                    let d = PyDict::new(py);
                    d.set_item("name", a.name.to_string())?;
                    d.set_item("type", a.r#type.clone())?;
                    d.set_item("ricardian_contract", a.ricardian_contract.clone())?;
                    ret.push(d);
                }
                Ok(ret)
            }

            #[getter]
            pub fn _tables<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
                let mut ret = Vec::new();
                for t in self.inner.tables.iter() {
                    let d = PyDict::new(py);
                    d.set_item("name", t.name_str())?;
                    d.set_item("key_names", t.key_names())?;
                    d.set_item("key_types", t.key_types())?;
                    d.set_item("index_type", t.index_type())?;
                    d.set_item("type", t.type_str())?;
                    ret.push(d);
                }
                Ok(ret)
            }

            pub fn resolve_type_into_dict<'py>(
                &self,
                py: Python<'py>,
                t: &str,
            ) -> PyResult<Bound<'py, PyDict>> {
                let res = self
                    .inner
                    .resolve_type(t)
                    .map_err(|e| PyTypeError::new_err(e.to_string()))?;

                let dict = PyDict::new(py);

                let is_struct = res
                    .is_struct
                    .map(|def| abi_struct_as_dict(py, &def))
                    .transpose()?;

                let is_variant = res
                    .is_variant
                    .map(|def| abi_variant_as_dict(py, &def))
                    .transpose()?;

                dict.set_item("original_name", res.original_name)?;
                dict.set_item("resolved_name", res.resolved_name)?;
                dict.set_item("is_std", res.is_std)?;
                dict.set_item("is_alias", res.is_alias)?;
                dict.set_item("is_variant", is_variant)?;
                dict.set_item("is_struct", is_struct)?;
                dict.set_item(
                    "modifiers".to_string(),
                    PyList::new(py, res.modifiers.iter().map(|tm| tm.as_str()))?,
                )?;

                Ok(dict)
            }

            pub fn pack<'py>(&self, t: &str, val: &Bound<'py, PyAny>) -> PyResult<Vec<u8>> {
                let mut encoder = Encoder::new(0);
                encode_abi_type(&self.inner, t, val, &mut encoder)?;
                Ok(encoder.get_bytes().to_vec())
            }

            pub fn unpack<'py>(
                &self,
                py: Python<'py>,
                t: &str,
                buf: &[u8],
            ) -> PyResult<Bound<'py, PyAny>> {
                let mut decoder = Decoder::new(buf);
                decode_abi_type(py, &self.inner, t, &mut decoder)
            }

            pub fn to_string(&self) -> String {
                let mut buf = Vec::new();
                let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                let mut ser = Serializer::with_formatter(&mut buf, fmt);
                self.inner.serialize(&mut ser).unwrap();
                String::from_utf8(buf).unwrap()
            }

            pub fn encode(&self) -> Vec<u8> {
                let mut encoder = Encoder::new(0);
                self.inner.pack(&mut encoder);
                encoder.get_bytes().to_vec()
            }

            fn __str__(&self) -> String {
                self.to_string()
            }

            fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
                match op {
                    CompareOp::Eq => Ok(self.inner == other.inner),
                    CompareOp::Ne => Ok(self.inner != other.inner),
                    _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                        "Operation not implemented",
                    )),
                }
            }
        }
    };
}

define_pyabi!(ABI, NativeABI);
define_pyabi!(ShipABI, NativeShipABI);
