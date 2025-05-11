use antelope::chain::abi::{ABI as NativeABI, ShipABI as NativeShipABI, ABITypeResolver, ABIResolvedType};
use antelope::serializer::{Decoder, Encoder, Packer};
use packvm::compiler::antelope::AntelopeSourceCode;
use packvm::{PackVM, RunTarget};
use packvm::compiler::SourceCode;
use packvm::utils::numbers::U48;
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Serializer;
use serde::ser::Serialize;
use crate::types::AntelopeValue;

fn resolved_type_to_dict(py: Python, resolved: (ABIResolvedType, String)) -> PyResult<Bound<PyDict>> {
    let dict = PyDict::new(py);
    match resolved {
        (ABIResolvedType::Standard(std), _) => {
            dict.set_item("type", "standard")?;
            dict.set_item("name", std)?;
        },
        (ABIResolvedType::Variant(meta), _) => {
            dict.set_item("type", "variant")?;
            dict.set_item("name", meta.name)?;
            dict.set_item("types", meta.types)?;
        }
        (ABIResolvedType::Struct(meta), _) => {
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
        (ABIResolvedType::Optional(_), name) => {
            dict.set_item("type", "modifier")?;
            dict.set_item("modifier", "optional")?;
            dict.set_item("type_name", name)?;
        }
        (ABIResolvedType::Array(_), name) => {
            dict.set_item("type", "modifier")?;
            dict.set_item("modifier", "array")?;
            dict.set_item("type_name", name)?;
        }
        (ABIResolvedType::Extension(_), name) => {
            dict.set_item("type", "modifier")?;
            dict.set_item("modifier", "extension")?;
            dict.set_item("type_name", name)?;
        }
    };
    Ok(dict)
}

macro_rules! define_pyabi {
    ($wrapper:ident, $inner:path) => {
        #[pyclass(unsendable)]
        #[derive(Debug, Clone)]
        pub struct $wrapper {
            pub inner: $inner,
            src: AntelopeSourceCode,
            vm:  PackVM,
        }

        impl $wrapper {
            fn bootstrap_vm(inner: &$inner) -> PyResult<(AntelopeSourceCode, PackVM)> {
                let src = AntelopeSourceCode::try_from(inner)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let ns  = packvm::compiler::compile_source(&src)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let exe = packvm::compiler::assemble(&ns)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok((src, PackVM::from_executable(exe)))
            }
        }

        #[pymethods]
        impl $wrapper {
            #[staticmethod]
            pub fn from_bytes(buf: &[u8]) -> PyResult<Self> {
                let mut decoder = Decoder::new(buf);
                let mut inner   = <$inner>::default();
                decoder.unpack(&mut inner)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let (src, vm) = Self::bootstrap_vm(&inner)?;
                Ok(Self { inner, src, vm })
            }

            #[staticmethod]
            pub fn from_str(s: &str) -> PyResult<Self> {
                let inner = <$inner>::from_string(s)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let (src, vm) = Self::bootstrap_vm(&inner)?;
                Ok(Self { inner, src, vm })
            }

            pub fn to_string(&self) -> String {
                let mut buf = Vec::new();
                let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                let mut ser = Serializer::with_formatter(&mut buf, fmt);
                self.inner.serialize(&mut ser).unwrap();
                String::from_utf8(buf).unwrap()
            }

            pub fn program_for(&self, type_name: &str) -> Option<RunTarget> {
                self.src.program_id_for(type_name)
            }

            #[pyo3(signature = (pid, value, modifier=None))]
            pub fn pack_direct(&mut self, pid: u64, value: AntelopeValue, modifier: Option<u8>) -> PyResult<Vec<u8>> {
                let m = if let Some(modifier) = modifier {
                    Some(modifier.into())
                } else {
                    None
                };
                let vm_val: packvm::Value = value.into();
                self.vm.run_pack(&RunTarget::new(U48(pid), m), &vm_val)
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            }

            pub fn pack_target(&mut self, target: RunTarget, value: AntelopeValue) -> PyResult<Vec<u8>> {
                let vm_val: packvm::Value = value.into();
                self.vm.run_pack(&target, &vm_val)
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            }

            pub fn pack(&mut self, type_alias: &str, value: AntelopeValue) -> PyResult<Vec<u8>> {
                let pid = self.src.program_id_for(type_alias)
                    .ok_or_else(|| PyValueError::new_err(format!("Program ID not found for {}", type_alias)))?;

                self.pack_target(pid, value)
            }

            #[pyo3(signature = (pid, buffer, modifier=None))]
            pub fn unpack_direct(&mut self, pid: u64, buffer: &[u8], modifier: Option<u8>) -> PyResult<AntelopeValue> {
                let m = if let Some(modifier) = modifier {
                    Some(modifier.into())
                } else {
                    None
                };
                self.vm.run_unpack(&RunTarget::new(U48(pid), m), buffer)
                    .map_err(|e| PyValueError::new_err(e.to_string()))
                    .map(|v| v.clone().into())
            }

            pub fn unpack_target(&mut self, target: RunTarget, buffer: &[u8]) -> PyResult<AntelopeValue> {
                self.vm.run_unpack(&target, buffer)
                    .map_err(|e| PyValueError::new_err(e.to_string()))
                    .map(|v| v.clone().into())
            }

            pub fn unpack(&mut self, type_alias: &str, buffer: &[u8]) -> PyResult<AntelopeValue> {
                let pid = self.src.program_id_for(type_alias)
                    .ok_or_else(|| PyValueError::new_err(format!("Program ID not found for {}", type_alias)))?;

                self.unpack_target(pid, buffer)
            }

            pub fn encode(&self) -> Vec<u8> {
                let mut encoder = Encoder::new(0);
                self.inner.pack(&mut encoder);
                encoder.get_bytes().to_vec()
            }

            pub fn resolve_type_into_dict<'py>(&self, py: Python<'py>, type_name: &str) -> PyResult<Option<Bound<'py, PyDict>>> {
                if let Some(resolved) = self.inner.resolve_type(type_name) {
                    return Ok(Some(resolved_type_to_dict(py, resolved)?))
                }
                Ok(None)
            }

            fn __str__(&self) -> String { self.to_string() }

            fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
                match op {
                    CompareOp::Eq => Ok(self.inner == other.inner),
                    CompareOp::Ne => Ok(self.inner != other.inner),
                    _ => Err(pyo3::exceptions::PyNotImplementedError::new_err("Operation not implemented")),
                }
            }
        }
    };
}

define_pyabi!(ABI, NativeABI);
define_pyabi!(ShipABI, NativeShipABI);
