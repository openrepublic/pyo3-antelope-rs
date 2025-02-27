use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::name::{Name as NativeName};
use pyo3::{Bound, FromPyObject, IntoPyObjectExt, Py, PyAny, PyErr};
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::types::{PyBytes, PyDict, PyFloat, PyInt, PyList};
use pyo3::prelude::*;
use crate::encode::encode_params;
use crate::proxies::asset::Asset;
use crate::proxies::name::Name;
use crate::proxies::sym::Symbol;
use crate::proxies::sym_code::SymbolCode;

#[derive(Debug)]
pub enum ActionDataTypes {
    Bool(bool),
    Int(Py<PyInt>),
    Float(Py<PyFloat>),
    Bytes(Vec<u8>),
    String(String),
    List(Py<PyList>),
    Struct(Py<PyDict>),
    SymbolCode(SymbolCode),
    Symbol(Symbol),
    Asset(Asset),
    Name(Name),
    None,
}

impl<'a> FromPyObject<'a> for ActionDataTypes {
    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        // Check if it's None
        if ob.is_none() {
            return Ok(ActionDataTypes::None);
        }

        // Try downcasting to bool
        if let Ok(py_bool) = ob.extract::<bool>() {
            return Ok(ActionDataTypes::Bool(py_bool));
        }

        // Try downcasting to PyInt
        if let Ok(py_int) = ob.downcast::<PyInt>() {
            return Ok(ActionDataTypes::Int(py_int.clone().unbind()));
        }

        // Try downcasting to PyFloat
        if let Ok(py_float) = ob.downcast::<PyFloat>() {
            return Ok(ActionDataTypes::Float(py_float.clone().unbind()));
        }

        // Try extracting as bytes (Python `bytes` or `bytearray`).
        //    ob.extract::<Vec<u8>>() handles both.
        if let Ok(byte_vec) = ob.downcast::<PyBytes>() {
            let byte_vec: Vec<u8> = byte_vec.extract()?;
            return Ok(ActionDataTypes::Bytes(byte_vec));
        }

        // Try extracting as a String
        if let Ok(s) = ob.extract::<String>() {
            return Ok(ActionDataTypes::String(s));
        }

        // Check if it's a list
        if let Ok(py_list) = ob.downcast::<PyList>() {
            return Ok(ActionDataTypes::List(py_list.clone().unbind()));
        }

        // Check if it's a dict
        if let Ok(py_dict) = ob.downcast::<PyDict>() {
            return Ok(ActionDataTypes::Struct(py_dict.clone().unbind()));
        }

        if let Ok(py_name) = ob.extract::<Name>() {
            return Ok(ActionDataTypes::Name(py_name));
        }

        if let Ok(py_sym_code) = ob.extract::<SymbolCode>() {
            return Ok(ActionDataTypes::SymbolCode(py_sym_code));
        }

        if let Ok(py_sym) = ob.extract::<Symbol>() {
            return Ok(ActionDataTypes::Symbol(py_sym));
        }

        if let Ok(py_asset) = ob.extract::<Asset>() {
            return Ok(ActionDataTypes::Asset(py_asset));
        }

        // If all attempts failed, raise a TypeError:
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            format!("Cannot convert to ActionDataTypes: {:?}", ob)
        ))
    }
}

impl<'py> IntoPyObject<'py> for ActionDataTypes {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            ActionDataTypes::Bool(val) => {
                Ok(val.into_bound_py_any(py)?)
            }
            ActionDataTypes::Int(int) => {
                Ok(int.into_bound_py_any(py)?)
            }
            ActionDataTypes::Float(val) => {
                Ok(val.into_bound_py_any(py)?)
            },
            ActionDataTypes::Bytes(val) => {
                Ok(val.into_bound_py_any(py)?)
            },
            ActionDataTypes::String(val) => {
                Ok(val.into_bound_py_any(py)?)
            },
            ActionDataTypes::List(list) => {
                Ok(list.into_bound_py_any(py)?)
            },
            ActionDataTypes::Struct(dict) => {
                Ok(dict.into_bound_py_any(py)?)
            },
            ActionDataTypes::SymbolCode(obj) => {
                Ok(obj.into_bound_py_any(py)?)
            },
            ActionDataTypes::Symbol(obj) => {
                Ok(obj.into_bound_py_any(py)?)
            },
            ActionDataTypes::Asset(obj) => {
                Ok(obj.into_bound_py_any(py)?)
            },
            ActionDataTypes::Name(obj) => {
                Ok(obj.into_bound_py_any(py)?)
            },
            ActionDataTypes::None => Ok(py.None().into_bound_py_any(py)?),
        }
    }
}

pub struct PyAction {
    pub account: String,
    pub name: String,
    pub authorization: Vec<PermissionLevel>,
    pub data: Vec<ActionDataTypes>,
}

pub fn into_action(action: &PyAction) -> PyResult<Action> {
    Ok(Action {
        account: NativeName::new_from_str(&action.account),
        name: NativeName::new_from_str(&action.name),
        authorization: action.authorization.clone(),
        data: encode_params(&action.account, &action.name, &action.data)?,
    })
}

impl<'a> FromPyObject<'a> for PyAction {
    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        // First, downcast to a PyDict or raise a TypeError if that fails.
        let py_dict = ob
            .downcast::<PyDict>()
            .map_err(|_| PyErr::new::<PyTypeError, _>("Expected a dict for PyAction"))?;

        // --- account ---
        let account_obj = py_dict
            .get_item("account")? // -> PyResult<Option<Bound<PyAny>>>
            .ok_or_else(|| PyErr::new::<PyKeyError, _>("Missing 'account' key"))?;
        let account: String = account_obj
            .extract()
            .map_err(|_| PyErr::new::<PyTypeError, _>("'account' must be a string"))?;

        // --- name ---
        let name_obj = py_dict
            .get_item("name")?
            .ok_or_else(|| PyErr::new::<PyKeyError, _>("Missing 'name' key"))?;
        let name: String = name_obj
            .extract()
            .map_err(|_| PyErr::new::<PyTypeError, _>("'name' must be a string"))?;

        // --- data ---
        let data_obj = py_dict
            .get_item("data")?
            .ok_or_else(|| PyErr::new::<PyKeyError, _>("Missing 'data' key"))?;
        let data: Vec<ActionDataTypes> = data_obj
            .extract()
            .map_err(|_| PyErr::new::<PyTypeError, _>("'data' must be a Vec<ActionDataTypes>"))?;

        // --- authorization ---
        let auth_obj = py_dict
            .get_item("authorization")?
            .ok_or_else(|| PyErr::new::<PyKeyError, _>("Missing 'authorization' key"))?;
        let auth_list = auth_obj
            .extract::<Vec<Bound<PyDict>>>()
            .map_err(|_| PyErr::new::<PyTypeError, _>("'authorization' must be a list of dicts"))?;

        // Convert each authorization dict into a PermissionLevel
        let mut authorization = Vec::with_capacity(auth_list.len());
        for auth_dict in auth_list {
            let actor_obj = auth_dict
                .get_item("actor")?
                .ok_or_else(|| PyErr::new::<PyKeyError, _>("Missing 'actor' key"))?;
            let actor_str: String = actor_obj
                .extract()
                .map_err(|_| PyErr::new::<PyTypeError, _>("'actor' must be a string"))?;
            let actor = NativeName::new_from_str(&actor_str);

            let perm_obj = auth_dict
                .get_item("permission")?
                .ok_or_else(|| PyErr::new::<PyKeyError, _>("Missing 'permission' key"))?;
            let perm_str: String = perm_obj
                .extract()
                .map_err(|_| PyErr::new::<PyTypeError, _>("'permission' must be a string"))?;
            let permission = NativeName::new_from_str(&perm_str);

            authorization.push(PermissionLevel { actor, permission });
        }

        Ok(PyAction {
            account,
            name,
            authorization,
            data,
        })
    }
}
