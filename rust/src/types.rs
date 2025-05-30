use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::name::Name as NativeName;
use pyo3::{FromPyObject, PyResult};
use pyo3::exceptions::PyValueError;

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

#[derive(FromPyObject)]
pub(crate) struct PyPermissionLevel {
    actor: String,
    permission: String
}

impl From<&PyPermissionLevel> for PyResult<PermissionLevel> {
    fn from(value: &PyPermissionLevel) -> Self {
        Ok(PermissionLevel::new(
            NativeName::try_from(&value.actor).map_err(|e| PyValueError::new_err(e.to_string()))?,
            NativeName::try_from(&value.permission).map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
}

#[derive(FromPyObject)]
pub(crate) struct PyAction {
    account: String,
    name: String,
    authorization: Vec<PyPermissionLevel>,
    data: Vec<u8>,
}

impl From<&PyAction> for PyResult<Action> {
    fn from(py_action: &PyAction) -> Self {
        let mut auths = Vec::new();
        for auth in py_action.authorization.iter() {
            let maybe_perm: PyResult<PermissionLevel> = auth.into();
            auths.push(maybe_perm?);
        }
        Ok(Action {
            account: NativeName::try_from(&py_action.account).map_err(|e| PyValueError::new_err(e.to_string()))?,
            name: NativeName::try_from(&py_action.name).map_err(|e| PyValueError::new_err(e.to_string()))?,
            authorization: auths,
            data: py_action.data.clone(),
        })
    }
}