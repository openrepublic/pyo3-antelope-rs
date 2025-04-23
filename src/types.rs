use std::collections::HashMap;
use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::name::{Name as NativeName};
use antelope::chain::asset::{
    Symbol as NativeSymbol,
    Asset as NativeAsset,
};
use antelope::serializer::generic::encode::encode_params;
use antelope::serializer::generic::value::{Value as NativeValue};
use pyo3::{Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3::prelude::*;
use pyo3_tools::py_struct;
use crate::proxies::abi::ABI;
use crate::proxies::asset::Asset;
use crate::proxies::name::Name;
use crate::proxies::sym::Symbol;
use crate::proxies::sym_code::SymbolCode;

py_struct!(PyPermissionLevel {
    actor: String,
    permission: String
});

impl Into<PyResult<PermissionLevel>> for PyPermissionLevel {
    fn into(self) -> PyResult<PermissionLevel> {
        Ok(PermissionLevel::new(
            NativeName::from_string(&self.actor).map_err(|e| PyValueError::new_err(e.to_string()))?,
            NativeName::from_string(&self.permission).map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
}

py_struct!(PyAction {
    account: String,
    name: String,
    authorization: Vec<PyPermissionLevel>,
    data: Vec<AntelopeTypes>,
});

impl PyAction {
    pub fn into_native(self, abi: &ABI) -> PyResult<Action> {
        let mut auths = Vec::new();
        for auth in self.authorization {
            let maybe_perm: PyResult<PermissionLevel> = auth.into();
            auths.push(maybe_perm?);
        }
        Ok(Action {
            account: NativeName::new_from_str(&self.account),
            name: NativeName::new_from_str(&self.name),
            authorization: auths,
            data: encode_params(
                &abi.inner,
                &self.name,
                &self.data.iter().map(|v| v.clone().into_value()).collect(),
            ).map_err(|e| PyValueError::new_err(e.to_string()))?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum AntelopeTypes {
    Value(NativeValue),
    SymbolCode(SymbolCode),
    Symbol(Symbol),
    Asset(Asset),
    Name(Name),
    ABI(ABI)
}

impl AntelopeTypes {
    pub fn into_value(self) -> NativeValue {
        match self {
            AntelopeTypes::Value(val) => val,
            AntelopeTypes::SymbolCode(val) => NativeValue::SymbolCode(val.inner.value()),
            AntelopeTypes::Symbol(val) => NativeValue::Symbol(val.inner.value()),
            AntelopeTypes::Asset(val) => NativeValue::Asset(val.amount(), val.symbol().inner.value()),
            AntelopeTypes::Name(val) => NativeValue::Name(val.inner.value()),
            AntelopeTypes::ABI(val) => NativeValue::Bytes(val.encode()),
        }
    }
}

fn value_from_pyany(obj: &Bound<PyAny>) -> PyResult<NativeValue> {
    // None → Null
    if obj.is_none() {
        return Ok(NativeValue::Null);
    }
    // bool
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(NativeValue::Bool(b));
    }
    // try the integer ranges first
    if let Ok(i) = obj.extract::<i8>()   { return Ok(NativeValue::Int8(i)); }
    if let Ok(i) = obj.extract::<i16>()  { return Ok(NativeValue::Int16(i)); }
    if let Ok(i) = obj.extract::<i32>()  { return Ok(NativeValue::Int32(i)); }
    if let Ok(i) = obj.extract::<i64>()  { return Ok(NativeValue::Int64(i)); }
    // final fallback to full i128
    if let Ok(big) = obj.extract::<i128>() {
        return Ok(NativeValue::Int128(big));
    }
    // unsigned
    if let Ok(u) = obj.extract::<u8>()   { return Ok(NativeValue::Uint8(u)); }
    if let Ok(u) = obj.extract::<u16>()  { return Ok(NativeValue::Uint16(u)); }
    if let Ok(u) = obj.extract::<u32>()  { return Ok(NativeValue::Uint32(u)); }
    if let Ok(u) = obj.extract::<u64>()  { return Ok(NativeValue::Uint64(u)); }
    if let Ok(bigu) = obj.extract::<u128>() {
        return Ok(NativeValue::Uint128(bigu));
    }
    // floats
    if let Ok(f) = obj.extract::<f32>()  { return Ok(NativeValue::Float32(f)); }
    if let Ok(f) = obj.extract::<f64>()  { return Ok(NativeValue::Float64(f)); }
    // bytes
    if let Ok(pybytes) = obj.downcast::<PyBytes>() {
        return Ok(NativeValue::Bytes(pybytes.as_bytes().to_vec()));
    }
    // string
    if let Ok(s) = obj.extract::<String>() {
        return Ok(NativeValue::String(s));
    }
    // array
    if let Ok(seq) = obj.downcast::<pyo3::types::PySequence>() {
        let mut vec = Vec::with_capacity(seq.len().unwrap_or(0) as usize);
        for idx in 0..seq.len()? {
            let item = seq.get_item(idx)?;
            vec.push(value_from_pyany(&item)?);
        }
        return Ok(NativeValue::Array(vec));
    }
    // dict → Struct
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = HashMap::new();
        for (k, v) in dict {
            let key: String = k.extract()?;
            map.insert(key, value_from_pyany(&v)?);
        }
        return Ok(NativeValue::Struct(map));
    }
    Err(PyErr::new::<PyTypeError, _>(
        format!("cannot convert Python type {} to NativeValue", obj.get_type()),
    ))
}

impl<'a> FromPyObject<'a> for AntelopeTypes {
    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        if let Ok(py_name) = ob.extract::<Name>() {
            return Ok(AntelopeTypes::Name(py_name));
        }

        if let Ok(py_sym_code) = ob.extract::<SymbolCode>() {
            return Ok(AntelopeTypes::SymbolCode(py_sym_code));
        }

        if let Ok(py_sym) = ob.extract::<Symbol>() {
            return Ok(AntelopeTypes::Symbol(py_sym));
        }

        if let Ok(py_asset) = ob.extract::<Asset>() {
            return Ok(AntelopeTypes::Asset(py_asset));
        }

        Ok(AntelopeTypes::Value(value_from_pyany(ob)?))
    }
}

impl<'py> IntoPyObject<'py> for AntelopeTypes {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            AntelopeTypes::Value(val) => {
               match val {
                   NativeValue::Null => Ok(py.None().into_bound_py_any(py)?),
                   NativeValue::Bool(b) => Ok(b.into_bound_py_any(py)?),
                   NativeValue::Int8(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Int16(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Int32(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Int64(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Int128(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Uint8(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Uint16(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Uint32(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Uint64(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Uint128(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Float32(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::Float64(num) => Ok(num.into_bound_py_any(py)?),
                   NativeValue::String(s) => Ok(s.into_bound_py_any(py)?),
                   NativeValue::Bytes(bytes) => Ok(bytes.into_bound_py_any(py)?),
                   NativeValue::Array(values) => {
                       let py_list = PyList::empty(py);
                       for val in values {
                           py_list.append(AntelopeTypes::Value(val).into_pyobject(py)?)?
                       }
                       Ok(py_list.into_any())
                   }
                   NativeValue::Struct(obj) => {
                       let py_dict = PyDict::new(py);
                       for (k, v) in obj {
                           py_dict.set_item(k, AntelopeTypes::Value(v))?
                       }
                       Ok(py_dict.into_any())
                   }
                   NativeValue::Name(name) => Ok(NativeName::from_u64(name).to_string().into_bound_py_any(py)?),
                   NativeValue::Symbol(sym) => Ok(NativeSymbol::from_value(sym).to_string().into_bound_py_any(py)?),
                   NativeValue::SymbolCode(sym_code) => Ok(NativeSymbol::from_value(sym_code).to_string().into_bound_py_any(py)?),
                   NativeValue::Asset(amount, sym) => Ok(NativeAsset::new(amount, NativeSymbol::from_value(sym)).to_string().into_bound_py_any(py)?),
               }
            },
            AntelopeTypes::SymbolCode(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::Symbol(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::Asset(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::Name(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::ABI(obj) => Ok(obj.into_bound_py_any(py)?),
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
            /// Build an instance from raw bytes.
            #[staticmethod]
            pub fn from_bytes(
                buffer: &[u8]
            ) -> ::pyo3::PyResult<Self>
            {
                let mut decoder = antelope::serializer::Decoder::new(buffer);
                let mut inner: $inner =
                    ::core::default::Default::default();
                decoder.unpack(&mut inner);
                Ok(Self { inner })
            }

            /// Encode the wrapped value back into bytes.
            pub fn encode(&self) -> ::std::vec::Vec<u8> {
                let mut encoder = antelope::serializer::Encoder::new(0);
                self.inner.pack(&mut encoder);
                encoder.get_bytes().to_vec()
            }

            // ---------- user‑supplied items ----------
            $($rest)*
        }
    };
}
