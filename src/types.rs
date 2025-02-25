use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::name::Name;
use pyo3::{Bound, FromPyObject, Py, PyAny, PyErr, PyResult};
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::types::{PyBytes, PyDict, PyFloat, PyInt, PyList};
use pyo3::prelude::*;
use crate::encode::encode_params;

#[derive(Debug)]
pub enum ActionDataTypes {
    Bool(bool),
    Int(Py<PyInt>),
    Float(Py<PyFloat>),
    Bytes(Vec<u8>),
    String(String),
    List(Py<PyList>),
    Struct(Py<PyDict>),
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

        // If all attempts failed, raise a TypeError:
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            format!("Cannot convert to ActionDataTypes: {:?}", ob)
        ))
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
        account: Name::new_from_str(&action.account),
        name: Name::new_from_str(&action.name),
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
            let actor = Name::new_from_str(&actor_str);

            let perm_obj = auth_dict
                .get_item("permission")?
                .ok_or_else(|| PyErr::new::<PyKeyError, _>("Missing 'permission' key"))?;
            let perm_str: String = perm_obj
                .extract()
                .map_err(|_| PyErr::new::<PyTypeError, _>("'permission' must be a string"))?;
            let permission = Name::new_from_str(&perm_str);

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
