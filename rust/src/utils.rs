use base64::{engine::general_purpose, Engine as _};
use hex;
use pyo3::PyResult;


pyo3::create_exception!(_lowlevel, BytesStringDecodeError, pyo3::exceptions::PyException);


/// Attempt decode first from base64, then hex
pub fn try_decode_string_bytes(s: &str, default_len: Option<usize>) -> PyResult<Vec<u8>> {
    if let Some(def_len) = default_len {
        if s == "0" {
            return Ok(vec![0; def_len]);
        }
    }
    match general_purpose::STANDARD.decode(s) {
        Ok(bytes) => Ok(bytes),
        Err(_) => match hex::decode(s) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(
                BytesStringDecodeError::new_err(
                    format!("Input is neither valid base64 nor hex: \"{s}\"")
                )
            )
        },
    }
}
