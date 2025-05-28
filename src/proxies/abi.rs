use antelope::chain::abi::{
    ABIResolvedType, ABITypeResolver, ShipABI as NativeShipABI, ABI as NativeABI, AbiTableView
};
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::ser::Serialize;
use serde_json::Serializer;

fn resolved_type_to_dict(
    py: Python,
    resolved: ABIResolvedType,
) -> PyResult<Bound<PyDict>> {
    let dict = PyDict::new(py);
    match resolved {
        ABIResolvedType::Standard(std) => {
            dict.set_item("type", "standard")?;
            dict.set_item("name", std)?;
        }
        ABIResolvedType::Variant(meta) => {
            dict.set_item("type", "variant")?;
            dict.set_item("name", meta.name)?;
            dict.set_item("types", meta.types)?;
        }
        ABIResolvedType::Struct(meta) => {
            dict.set_item("type", "struct")?;
            dict.set_item("name", meta.name)?;
            dict.set_item("base", meta.base)?;
            let mut fields = Vec::with_capacity(meta.fields.len());
            for f in meta.fields.iter() {
                let field = PyDict::new(py);
                field.set_item("name", f.name.clone())?;
                field.set_item("type", f.r#type.clone())?;
                fields.push(field);
            }
            dict.set_item("fields", fields)?;
        }
        ABIResolvedType::Alias(inner) => {
            dict.set_item("type", "alias")?;
            dict.set_item("inner", resolved_type_to_dict(py, *inner)?)?;
        }
        ABIResolvedType::Optional(inner) => {
            dict.set_item("type", "optional")?;
            dict.set_item("inner", resolved_type_to_dict(py, *inner)?)?;
        }
        ABIResolvedType::Extension(inner) => {
            dict.set_item("type", "extension")?;
            dict.set_item("inner", resolved_type_to_dict(py, *inner)?)?;
        }
        ABIResolvedType::Array(inner) => {
            dict.set_item("type", "array")?;
            dict.set_item("item", resolved_type_to_dict(py, *inner)?)?;
        }
    };
    Ok(dict)
}

macro_rules! define_pyabi {
    ($wrapper:ident, $inner:path) => {
        #[pyclass]
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
            pub fn from_str(s: &str) -> PyResult<Self> {
                let inner =
                    <$inner>::from_string(s).map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(Self { inner })
            }

            #[getter]
            pub fn version(&self) -> &String {
                &self.inner.version
            }

            #[getter]
            pub fn types<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
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
            pub fn structs<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
                let mut ret = Vec::new();
                for s in self.inner.structs.iter() {
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
                    ret.push(d);
                }
                Ok(ret)
            }

            #[getter]
            pub fn variants<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
                let mut ret = Vec::new();
                for v in self.inner.variants.iter() {
                    let d = PyDict::new(py);
                    d.set_item("name", v.name.clone())?;
                    d.set_item("types", v.types.clone())?;
                    ret.push(d);
                }
                Ok(ret)
            }

            #[getter]
            pub fn actions<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
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
            pub fn tables<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
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

            pub fn resolve_type<'py>(
                &self,
                py: Python<'py>,
                type_name: &str,
            ) -> PyResult<Bound<'py, PyDict>> {
                let resolved = self.inner.resolve_type(type_name).map_err(|e| PyValueError::new_err(e.to_string()))?;
                return Ok(resolved_type_to_dict(py, resolved)?);
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
