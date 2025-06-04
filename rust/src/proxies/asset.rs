use antelope::chain::asset::{Asset, ExtendedAsset};
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rust_decimal::Decimal;
use std::fmt::Display;
use std::str::FromStr;

use crate::proxies::{
    name::{PyName, NameLike},
    sym::{PySymbol, SymLike},
};

#[pyclass(frozen, name = "Asset")]
#[derive(Debug, Clone)]
pub struct PyAsset {
    pub inner: Asset,
}

#[derive(FromPyObject)]
pub enum AssetLike<'py> {
    Raw([u8; 16]),
    Str(String),
    Int(i64, SymLike),
    Decimal(Decimal, SymLike),
    Dict(Bound<'py, PyDict>),
    Cls(PyAsset),
}

impl From<PyAsset> for Asset {
    fn from(value: PyAsset) -> Self {
        value.inner
    }
}

impl From<Asset> for PyAsset {
    fn from(value: Asset) -> Self {
        PyAsset { inner: value }
    }
}

#[pymethods]
impl PyAsset {
    #[new]
    fn new(amount: i64, sym: SymLike) -> PyResult<Self> {
        let sym = PySymbol::try_from(sym)?;
        Asset::try_from((amount, sym.inner))
            .map(|a| a.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: Asset = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        Asset::from_str(s)
            .map(|a| a.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn from_decimal(d: Decimal, sym: SymLike) -> PyResult<Self> {
        let sym = PySymbol::try_from(sym)?;

        let d_str = d.to_string().replace(".", "");
        let amount = i64::from_str(&d_str)
            .map_err(|e| PyValueError::new_err(format!("Decimal not valid i64: {e}")))?;

        PyAsset::new(amount, SymLike::Cls(sym))
    }

    #[staticmethod]
    pub fn from_dict<'py>(d: Bound<'py, PyDict>) -> PyResult<Self> {
        let py_amount = d
            .get_item("amount")?
            .ok_or(PyKeyError::new_err(
                "Expected asset dict to have amount key",
            ))?
            .extract()?;

        let py_symbol = d
            .get_item("symbol")?
            .ok_or(PyKeyError::new_err(
                "Expected asset dict to have amount key",
            ))?
            .extract()?;

        PyAsset::new(py_amount, py_symbol)
    }

    #[staticmethod]
    pub fn try_from<'py>(value: AssetLike<'py>) -> PyResult<PyAsset> {
        match value {
            AssetLike::Raw(raw) => PyAsset::from_bytes(&raw),
            AssetLike::Str(s) => PyAsset::from_str_py(&s),
            AssetLike::Int(amount, sym) => PyAsset::new(amount, sym),
            AssetLike::Decimal(d, sym) => PyAsset::from_decimal(d, sym),
            AssetLike::Dict(d) => PyAsset::from_dict(d),
            AssetLike::Cls(asset) => Ok(asset),
        }
    }

    fn to_decimal(&self) -> Decimal {
        let mut str_amount = format!(
            "{:0>width$}",
            self.amount(),
            width = (self.symbol().precision() + 1) as usize
        );

        if self.symbol().precision() > 0 {
            let len = str_amount.len();
            str_amount.insert(len - self.symbol().precision() as usize, '.');
        }

        Decimal::from_str(&str_amount).unwrap_or(Decimal::ZERO)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    #[getter]
    pub fn amount(&self) -> i64 {
        self.inner.amount()
    }

    #[getter]
    pub fn symbol(&self) -> PySymbol {
        PySymbol {
            inner: self.inner.symbol(),
        }
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<PyAsset>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }

    fn __add__(&self, other: &PyAsset) -> PyResult<PyAsset> {
        let result = self
            .inner
            .try_add(other.inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyAsset { inner: result })
    }

    fn __sub__(&self, other: &PyAsset) -> PyResult<PyAsset> {
        let result = self
            .inner
            .try_sub(other.inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyAsset { inner: result })
    }
}

impl Display for PyAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass(frozen, name = "ExtendedAsset")]
#[derive(Debug, Clone)]
pub struct PyExtendedAsset {
    pub inner: ExtendedAsset,
}

#[derive(FromPyObject)]
pub enum ExtAssetLike<'py> {
    Raw([u8; 24]),
    Str(String),
    Dict(Bound<'py, PyDict>),
    Cls(PyExtendedAsset),
}

impl From<PyExtendedAsset> for ExtendedAsset {
    fn from(value: PyExtendedAsset) -> Self {
        ExtendedAsset {
            quantity: value.inner.quantity,
            contract: value.inner.contract,
        }
    }
}

impl From<ExtendedAsset> for PyExtendedAsset {
    fn from(value: ExtendedAsset) -> Self {
        PyExtendedAsset { inner: value }
    }
}

#[pymethods]
impl PyExtendedAsset {
    #[staticmethod]
    pub fn from_bytes(buffer: &[u8]) -> ::pyo3::PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: ExtendedAsset = Default::default();
        decoder
            .unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        let ext = ExtendedAsset::from_str(s).map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(ext.into())
    }

    #[staticmethod]
    pub fn from_dict<'py>(d: Bound<'py, PyDict>) -> PyResult<Self> {
        let quantity = PyAsset::try_from(
            d.get_item("quantity")?
                .ok_or(PyKeyError::new_err(
                    "Expected asset dict to have amount key",
                ))?
                .extract::<AssetLike>()?,
        )?;

        let contract = PyName::try_from(
            d.get_item("contract")?
                .ok_or(PyKeyError::new_err(
                    "Expected asset dict to have amount key",
                ))?
                .extract::<NameLike>()?,
        )?;

        Ok(ExtendedAsset {
            quantity: quantity.inner,
            contract: contract.inner,
        }
        .into())
    }

    #[staticmethod]
    pub fn try_from<'py>(value: ExtAssetLike<'py>) -> PyResult<PyExtendedAsset> {
        match value {
            ExtAssetLike::Raw(raw) => PyExtendedAsset::from_bytes(&raw),
            ExtAssetLike::Str(s) => PyExtendedAsset::from_str_py(&s),
            ExtAssetLike::Dict(d) => PyExtendedAsset::from_dict(d),
            ExtAssetLike::Cls(ext_asset) => Ok(ext_asset),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::new(0);
        self.inner.pack(&mut encoder);
        encoder.get_bytes().to_vec()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __add__(&self, other: &PyExtendedAsset) -> PyResult<PyExtendedAsset> {
        let result = self
            .inner
            .quantity
            .try_add(other.inner.quantity)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(ExtendedAsset {
            quantity: result,
            contract: other.inner.contract,
        }
        .into())
    }

    fn __sub__(&self, other: &PyExtendedAsset) -> PyResult<PyExtendedAsset> {
        let result = self
            .inner
            .quantity
            .try_sub(other.inner.quantity)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(ExtendedAsset {
            quantity: result,
            contract: other.inner.contract,
        }
        .into())
    }
}

impl Display for PyExtendedAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
