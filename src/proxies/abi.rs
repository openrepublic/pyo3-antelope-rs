use antelope::chain::abi::ABI as NativeABI;
use antelope::serializer::{Decoder, Encoder, Packer};
use antelope::serializer::generic::decode::decode_abi_type;
use antelope::serializer::generic::encode::encode_abi_type;
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde_json::Serializer;
use serde::ser::Serialize;
use crate::impl_packable_py;
use crate::types::AntelopeTypes;

/// A Python-exposed wrapper around the `antelope::Name` struct.
#[pyclass]
#[derive(Debug, Clone)]
pub struct ABI {
    pub inner: NativeABI,
}

impl_packable_py! {
    impl ABI(NativeABI) {
        #[staticmethod]
        pub fn from_str(s: &str) -> PyResult<Self> {
            let abi = NativeABI::from_string(s).map_err(|e| PyValueError::new_err(e.to_string()))?;

            Ok(ABI {
                inner: abi,
            })
        }

        pub fn to_string(&self) -> String {
            let mut buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    "); // 4 spaces
            let mut serializer = Serializer::with_formatter(&mut buf, formatter);

            self.inner.serialize(&mut serializer).unwrap();

            String::from_utf8(buf).unwrap()
        }

        pub fn pack_self(&self) -> Vec<u8> {
            self.encode()
        }

        pub fn pack(&self, type_alias: &str, value: AntelopeTypes) -> PyResult<Vec<u8>> {
            let mut encoder = Encoder::new(0);

            encode_abi_type(&self.inner, type_alias, &value.into_value(), &mut encoder)
                .map_err(|err| PyValueError::new_err(err.to_string()))?;

            Ok(encoder.get_bytes().to_vec())
        }

        pub fn unpack(&self, type_alias: &str, buffer: &[u8]) -> PyResult<AntelopeTypes> {
            let mut decoder = Decoder::new(buffer);

            Ok(AntelopeTypes::Value(decode_abi_type(&self.inner, type_alias, buffer.len(), &mut decoder)
                .map_err(|err| PyValueError::new_err(err.to_string()))?))
        }

        fn __str__(&self) -> String { self.to_string() }

        fn __richcmp__(&self, other: &ABI, op: CompareOp) -> PyResult<bool> {
            match op {
                CompareOp::Eq => Ok(self.inner == other.inner),
                CompareOp::Ne => Ok(self.inner != other.inner),
                _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                    "Operation not implemented",
                )),
            }
        }
    }
}