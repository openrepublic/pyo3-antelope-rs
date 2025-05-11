use std::collections::HashMap;
use packvm::Value;
use pyo3::{Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr};
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3::prelude::*;
use crate::proxies::abi::ABI;
use crate::proxies::asset::{Asset, ExtendedAsset};
use crate::proxies::name::Name;
use crate::proxies::sym::Symbol;
use crate::proxies::sym_code::SymbolCode;

// import packvm antelope conversion traits
#[allow(unused_imports)]
use packvm::compiler::antelope as vmantelope;
use crate::proxies::checksums::{Checksum160, Checksum256, Checksum512};
use crate::proxies::public_key::PublicKey;

#[derive(Debug, Clone)]
pub enum AntelopeTypes {
    Sum160(Checksum160),
    Sum256(Checksum256),
    Sum512(Checksum512),

    PublicKey(PublicKey),

    SymbolCode(SymbolCode),
    Symbol(Symbol),
    Asset(Asset),
    ExtendedAsset(ExtendedAsset),

    Name(Name),

    ABI(ABI),
}

#[derive(Debug, Clone)]
pub enum AntelopeValue {
    Generic(Value),
    Antelope(AntelopeTypes),
    List(Vec<AntelopeValue>),
    Dict(HashMap<String, AntelopeValue>),
}

impl Default for AntelopeValue {
    fn default() -> Self {
        AntelopeValue::Generic(Value::None)
    }
}

impl From<Value> for AntelopeValue {
    fn from(value: Value) -> Self {
        AntelopeValue::Generic(value)
    }
}

impl From<AntelopeTypes> for Value {
    fn from(value: AntelopeTypes) -> Self {
        match value {
            AntelopeTypes::Name(wrapper) => wrapper.inner.into(),

            AntelopeTypes::Sum160(wrapper) => wrapper.inner.into(),
            AntelopeTypes::Sum256(wrapper) => wrapper.inner.into(),
            AntelopeTypes::Sum512(wrapper) => wrapper.inner.into(),

            AntelopeTypes::PublicKey(wrapper) => wrapper.inner.into(),

            AntelopeTypes::SymbolCode(wrapper) => wrapper.inner.into(),
            AntelopeTypes::Symbol(wrapper) => wrapper.inner.into(),
            AntelopeTypes::Asset(wrapper) => wrapper.inner.into(),
            AntelopeTypes::ExtendedAsset(wrapper) => {
                Value::Struct(HashMap::from([
                    ("quantity".to_string(), wrapper.quantity.inner.into()),
                    ("contract".to_string(), wrapper.contract.inner.into()),
                ]))
            }

            AntelopeTypes::ABI(wrapper) => wrapper.inner.into(),
        }
    }
}

impl From<AntelopeValue> for Value {
    fn from(value: AntelopeValue) -> Self {
        match value {
            AntelopeValue::Generic(value) => value,
            AntelopeValue::Antelope(wrapper) => wrapper.into(),
            AntelopeValue::List(list) => {
                Value::Array(list.into_iter().map(|v| v.into()).collect())
            },
            AntelopeValue::Dict(dict) => {
                Value::Struct(dict.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

impl<'a> FromPyObject<'a> for AntelopeValue {
    fn extract_bound(obj: &Bound<'a, PyAny>) -> PyResult<Self> {
        // None
        if obj.is_none() {
            return Ok(
                AntelopeValue::Generic(Value::None)
            );
        }
        // bool
        if let Ok(b) = obj.extract::<bool>() {
            return Ok(AntelopeValue::Generic(Value::Bool(b)));
        }
        if let Ok(u) = obj.extract::<u64>() {
            return Ok(AntelopeValue::Generic(u.into()));
        }
        if let Ok(i) = obj.extract::<i64>() {
            return Ok(AntelopeValue::Generic(i.into()));
        }
        // final fallback to full 128
        if let Ok(bigu) = obj.extract::<u128>() {
            return Ok(AntelopeValue::Generic(bigu.into()));
        }
        if let Ok(big) = obj.extract::<i128>() {
            return Ok(AntelopeValue::Generic(big.into()));
        }
        // floats
        if let Ok(f) = obj.extract::<f64>() {
            return Ok(AntelopeValue::Generic(Value::Float(f.into())));
        }
        // bytes
        if let Ok(pybytes) = obj.downcast::<PyBytes>() {
            return Ok(
                AntelopeValue::Generic(Value::Bytes(pybytes.as_bytes().to_vec()))
            );
        }
        // string
        if let Ok(s) = obj.extract::<String>() {
            return Ok(AntelopeValue::Generic(Value::String(s)));
        }
        // array
        if let Ok(seq) = obj.downcast::<pyo3::types::PySequence>() {
            let seq_len = seq.len().unwrap_or_default();
            let mut list = Vec::with_capacity(seq_len);
            for i in 0..seq_len {
                let item = seq.get_item(i)?;
                list.push(item.extract::<AntelopeValue>()?);
            }
            return Ok(AntelopeValue::List(list));
        }
        // dict
        if let Ok(dict) = obj.downcast::<PyDict>() {
            let mut map = HashMap::new();
            for (k, v) in dict {
                let key: String = k.extract()?;
                let value: AntelopeValue = v.extract()?;
                map.insert(key, value);
            }
            return Ok(AntelopeValue::Dict(map));
        }
        // py wrappers
        if let Ok(wrapper) = obj.extract::<AntelopeTypes>() {
            return Ok(AntelopeValue::Antelope(wrapper));
        }
        Err(PyTypeError::new_err(
            format!("cannot convert Python type {} to NativeValue", obj.get_type()),
        ))
    }
}

impl<'a> FromPyObject<'a> for AntelopeTypes {
    fn extract_bound(obj: &Bound<'a, PyAny>) -> PyResult<Self> {
        // py wrappers
        if let Ok(py_name) = obj.extract::<Name>() {
            return Ok(AntelopeTypes::Name(py_name));
        }

        if let Ok(py_key) = obj.extract::<PublicKey>() {
            return Ok(AntelopeTypes::PublicKey(py_key));
        }

        if let Ok(py_sym_code) = obj.extract::<SymbolCode>() {
            return Ok(AntelopeTypes::SymbolCode(py_sym_code));
        }

        if let Ok(py_sym) = obj.extract::<Symbol>() {
            return Ok(AntelopeTypes::Symbol(py_sym));
        }

        if let Ok(py_asset) = obj.extract::<Asset>() {
            return Ok(AntelopeTypes::Asset(py_asset));
        }

        if let Ok(py_ext_asset) = obj.extract::<ExtendedAsset>() {
            return Ok(AntelopeTypes::ExtendedAsset(py_ext_asset));
        }

        if let Ok(py_sum_160) = obj.extract::<Checksum160>() {
            return Ok(AntelopeTypes::Sum160(py_sum_160));
        }

        if let Ok(py_sum_256) = obj.extract::<Checksum256>() {
            return Ok(AntelopeTypes::Sum256(py_sum_256));
        }

        if let Ok(py_sum_512) = obj.extract::<Checksum512>() {
            return Ok(AntelopeTypes::Sum512(py_sum_512));
        }

        Err(PyTypeError::new_err(
            format!("cannot convert Python type {} to AntelopeTypes", obj.get_type()),
        ))
    }
}

impl<'py> IntoPyObject<'py> for AntelopeTypes {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            AntelopeTypes::Name(wrapper) => wrapper.into_bound_py_any(py),

            AntelopeTypes::Sum160(wrapper) => wrapper.into_bound_py_any(py),
            AntelopeTypes::Sum256(wrapper) => wrapper.into_bound_py_any(py),
            AntelopeTypes::Sum512(wrapper) => wrapper.into_bound_py_any(py),

            AntelopeTypes::PublicKey(wrapper) => wrapper.into_bound_py_any(py),

            AntelopeTypes::SymbolCode(wrapper) => wrapper.into_bound_py_any(py),
            AntelopeTypes::Symbol(wrapper) => wrapper.into_bound_py_any(py),
            AntelopeTypes::Asset(wrapper) => wrapper.into_bound_py_any(py),
            AntelopeTypes::ExtendedAsset(wrapper) => wrapper.into_bound_py_any(py),

            AntelopeTypes::ABI(wrapper) => wrapper.into_bound_py_any(py),
        }
    }
}

impl<'py> IntoPyObject<'py> for AntelopeValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            AntelopeValue::Generic(val) => val.into_bound_py_any(py),
            AntelopeValue::Antelope(wrapper) => wrapper.into_pyobject(py),
            AntelopeValue::List(list) => {
                let py_list = PyList::empty(py);
                for val in list {
                    py_list.append(val.into_pyobject(py)?)?
                }
                Ok(py_list.into_any())
            },
            AntelopeValue::Dict(dict) => {
                let py_dict = PyDict::new(py);
                for (k, v) in dict {
                    py_dict.set_item(k, v)?
                }
                Ok(py_dict.into_any())
            }
        }
    }
}


#[macro_export]
macro_rules! impl_packable_py {
    (
        impl $wrapper:ident ( $inner:ty ) {
            $($rest:tt)*
        }
    ) => {
        #[pymethods]
        impl $wrapper {
            // build an instance from raw bytes.
            #[staticmethod]
            pub fn from_bytes(
                buffer: &[u8]
            ) -> ::pyo3::PyResult<Self>
            {
                let mut decoder = ::antelope::serializer::Decoder::new(buffer);
                let mut inner: $inner =
                    ::core::default::Default::default();
                decoder.unpack(&mut inner)
                    .map_err(|e| ::pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(Self { inner })
            }

            // encode the wrapped value back into bytes.
            pub fn encode(&self) -> ::std::vec::Vec<u8> {
                let mut encoder = ::antelope::serializer::Encoder::new(0);
                self.inner.pack(&mut encoder);
                encoder.get_bytes().to_vec()
            }

            $($rest)*
        }
    };
}
