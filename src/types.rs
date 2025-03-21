use std::str::FromStr;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::name::{Name as NativeName};
use antelope::serializer::serde::encode::encode_params;
use pyo3::{Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr};
use pyo3::exceptions::{PyKeyError, PyTypeError, PyValueError};
use pyo3::types::{PyBytes, PyDict, PyFloat, PyInt, PyList};
use pyo3::prelude::*;
use pyo3_tools::py_struct;
use serde_json::{Map, Number, Value};
use crate::abi_store::get_abi;
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

impl Into<PyResult<Action>> for PyAction {
    fn into(self) -> PyResult<Action> {
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
                &get_abi(&self.account)
                    .map_err(|e| PyKeyError::new_err(e))?,
                &self.account,
                &self.name,
                &self.data.iter().map(|v| v.clone().into_value()).collect(),
            ).map_err(|e| PyValueError::new_err(e.to_string()))?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum AntelopeTypes {
    Value(Value),
    Bytes(Vec<u8>),
    SymbolCode(SymbolCode),
    Symbol(Symbol),
    Asset(Asset),
    Name(Name),
}

impl AntelopeTypes {
    pub fn into_value(self) -> Value {
        match self {
            AntelopeTypes::Value(val) => val,
            AntelopeTypes::Bytes(val) => Value::String(BASE64_STANDARD.encode(&val)),
            AntelopeTypes::SymbolCode(val) => Value::Number(Number::from(val.inner.value())),
            AntelopeTypes::Symbol(val) => Value::Number(Number::from(val.inner.value())),
            AntelopeTypes::Asset(val) => Value::String(val.inner.to_string()),
            AntelopeTypes::Name(val) => Value::Number(Number::from(val.inner.value())),
        }
    }
}

impl<'a> FromPyObject<'a> for AntelopeTypes {
    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        // Check if it's None
        if ob.is_none() {
            return Ok(AntelopeTypes::Value(Value::Null));
        }

        // Try downcasting to bool
        if let Ok(py_bool) = ob.extract::<bool>() {
            return Ok(AntelopeTypes::Value(Value::Bool(py_bool)));
        }

        // Try downcasting to PyInt
        if let Ok(py_int) = ob.downcast::<PyInt>() {
            let num_str = py_int.to_string();
            return Ok(AntelopeTypes::Value(Value::Number(
                Number::from_str(&num_str)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?)));
        }

        // Try downcasting to PyFloat
        if let Ok(py_float) = ob.downcast::<PyFloat>() {
            let f: f64 = py_float.extract()?;
            return Ok(AntelopeTypes::Value(Value::Number(Number::from_f64(f)
                .ok_or_else(|| PyValueError::new_err("f64 cannot be converted to Number"))?)));
        }

        // Try extracting as bytes (Python `bytes` or `bytearray`).
        //    ob.extract::<Vec<u8>>() handles both.
        if let Ok(byte_vec) = ob.downcast::<PyBytes>() {
            let byte_vec: Vec<u8> = byte_vec.extract()?;
            return Ok(AntelopeTypes::Bytes(byte_vec));
        }

        // Try extracting as a String
        if let Ok(s) = ob.extract::<String>() {
            return Ok(AntelopeTypes::Value(Value::String(s)));
        }

        // Check if it's a list
        if let Ok(py_list) = ob.downcast::<PyList>() {
            let mut values: Vec<Value> = Vec::with_capacity(py_list.len());
            for item in py_list {
                let value = AntelopeTypes::extract_bound(&item)?
                    .into_value();
                values.push(value);
            }
            return Ok(AntelopeTypes::Value(Value::Array(values)));
        }

        // Check if it's a dict
        if let Ok(py_dict) = ob.downcast::<PyDict>() {
            let mut obj: Map<String, Value> = Map::new();
            for (k, v) in py_dict {
                let key: String = k.extract()?;
                let value: Value = AntelopeTypes::extract_bound(&v)?
                    .into_value();
                obj.insert(key, value);
            }
            return Ok(AntelopeTypes::Value(Value::Object(obj)));
        }

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

        // If all attempts failed, raise a TypeError:
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            format!("Cannot convert to ActionDataTypes: {:?}", ob)
        ))
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
                   Value::Null => Ok(py.None().into_bound_py_any(py)?),
                   Value::Bool(b) => Ok(b.into_bound_py_any(py)?),
                   Value::Number(num) => {
                       if num.is_f64() {
                           return Ok(num.as_f64().unwrap().into_bound_py_any(py)?)
                       } else if num.is_u64() {
                           return Ok(num.as_u64().unwrap().into_bound_py_any(py)?)
                       } else if num.is_i64() {
                           return Ok(num.as_i64().unwrap().into_bound_py_any(py)?)
                       }
                       Err(PyValueError::new_err("Cannot convert Value::Number into py bound"))
                   },
                   Value::String(s) => Ok(s.into_bound_py_any(py)?),
                   Value::Array(values) => {
                       let py_list = PyList::empty(py);
                       for val in values {
                           py_list.append(AntelopeTypes::Value(val).into_pyobject(py)?)?
                       }
                       Ok(py_list.into_any())
                   }
                   Value::Object(obj) => {
                       let py_dict = PyDict::new(py);
                       for (k, v) in obj {
                           py_dict.set_item(k, AntelopeTypes::Value(v))?
                       }
                       Ok(py_dict.into_any())
                   }
               }
            },
            AntelopeTypes::SymbolCode(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::Symbol(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::Asset(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::Name(obj) => Ok(obj.into_bound_py_any(py)?),
            AntelopeTypes::Bytes(bytes) => Ok(bytes.into_bound_py_any(py)?)
        }
    }
}
