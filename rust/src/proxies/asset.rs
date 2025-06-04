use antelope::chain::asset::{
    Asset as NativeAsset, ExtendedAsset as NativeExtAsset,
};
use antelope::serializer::{Decoder, Encoder, Packer};
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rust_decimal::Decimal;
use std::fmt::Display;
use std::str::FromStr;

use crate::proxies::name::Name;
use crate::proxies::sym::Symbol;
use crate::proxies::name::NameLike;
use crate::proxies::sym::SymLike;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Asset {
    pub inner: NativeAsset,
}

#[derive(FromPyObject)]
pub enum AssetLike<'py> {
    Raw([u8; 16]),
    Str(String),
    Int(i64, SymLike),
    Decimal(Decimal, SymLike),
    Dict(Bound<'py, PyDict>),
    Cls(Asset)
}

impl From<Asset> for NativeAsset {
    fn from(value: Asset) -> Self {
        value.inner
    }
}

impl From<NativeAsset> for Asset {
    fn from(value: NativeAsset) -> Self {
        Asset { inner: value }
    }
}

#[pymethods]
impl Asset {
    #[new]
    fn new(amount: i64, sym: SymLike) -> PyResult<Self> {
        let sym = Symbol::try_from(sym)?;
        NativeAsset::try_from((amount, sym.inner))
            .map(|a| a.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn from_bytes(
        buffer: &[u8]
    ) -> PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: NativeAsset = Default::default();
        decoder.unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        NativeAsset::from_str(s)
            .map(|a| a.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn from_decimal(d: Decimal, sym: SymLike) -> PyResult<Self> {
        let sym = Symbol::try_from(sym)?;

        let d_str = d.to_string().replace(".", "");
        let amount = i64::from_str(&d_str)
            .map_err(|e| PyValueError::new_err(format!("Decimal not valid i64: {e}")))?;

        Asset::new(amount, SymLike::Cls(sym))
    }

    #[staticmethod]
    pub fn from_dict<'py>(d: Bound<'py, PyDict>) -> PyResult<Self> {
        let py_amount = d.get_item("amount")?
            .ok_or(PyKeyError::new_err("Expected asset dict to have amount key"))?
            .extract()?;

        let py_symbol = d.get_item("symbol")?
            .ok_or(PyKeyError::new_err("Expected asset dict to have amount key"))?
            .extract()?;

        Asset::new(py_amount, py_symbol)
    }

    #[staticmethod]
    pub fn try_from<'py>(value: AssetLike<'py>) -> PyResult<Asset> {
        match value {
            AssetLike::Raw(raw) => Asset::from_bytes(&raw),
            AssetLike::Str(s) => Asset::from_str_py(&s),
            AssetLike::Int(amount, sym) => Asset::new(amount, sym),
            AssetLike::Decimal(d, sym) => Asset::from_decimal(d, sym),
            AssetLike::Dict(d) => Asset::from_dict(d),
            AssetLike::Cls(asset) => Ok(asset)
        }
    }

    fn to_decimal(&self) -> Decimal {
        let mut str_amount = format!("{:0>width$}", self.amount(), width = (self.symbol().precision() + 1) as usize);

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
    pub fn symbol(&self) -> Symbol {
        Symbol {
            inner: self.inner.symbol(),
        }
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __richcmp__(&self, other: PyRef<Asset>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Operation not implemented",
            )),
        }
    }

    fn __add__(&self, other: &Asset) -> PyResult<Asset> {
        let result = self.inner.try_add(other.inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Asset { inner: result })
    }

    fn __sub__(&self, other: &Asset) -> PyResult<Asset> {
        let result = self.inner.try_sub(other.inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Asset { inner: result })
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ExtendedAsset {
    pub inner: NativeExtAsset
}

#[derive(FromPyObject)]
pub enum ExtAssetLike<'py> {
    Raw([u8; 24]),
    Str(String),
    Dict(Bound<'py, PyDict>),
    Cls(ExtendedAsset)
}

impl From<ExtendedAsset> for NativeExtAsset {
    fn from(value: ExtendedAsset) -> Self {
        NativeExtAsset {
            quantity: value.inner.quantity,
            contract: value.inner.contract,
        }
    }
}

impl From<NativeExtAsset> for ExtendedAsset {
    fn from(value: NativeExtAsset) -> Self {
        ExtendedAsset { inner: value }
    }
}

#[pymethods]
impl ExtendedAsset {
    #[staticmethod]
    pub fn from_bytes(
        buffer: &[u8]
    ) -> ::pyo3::PyResult<Self> {
        let mut decoder = Decoder::new(buffer);
        let mut inner: NativeExtAsset = Default::default();
        decoder.unpack(&mut inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(inner.into())
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    pub fn from_str_py(s: &str) -> PyResult<Self> {
        let ext = NativeExtAsset::from_str(s)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(ext.into())
    }

    #[staticmethod]
    pub fn from_dict<'py>(d: Bound<'py, PyDict>) -> PyResult<Self> {
        let quantity = Asset::try_from(
            d.get_item("quantity")?
                .ok_or(PyKeyError::new_err("Expected asset dict to have amount key"))?
                .extract::<AssetLike>()?
        )?;

        let contract = Name::try_from(
            d.get_item("contract")?
                .ok_or(PyKeyError::new_err("Expected asset dict to have amount key"))?
                .extract::<NameLike>()?
        )?;

        Ok(NativeExtAsset{
            quantity: quantity.inner, contract: contract.inner
        }.into())
    }

    #[staticmethod]
    pub fn try_from<'py>(value: ExtAssetLike<'py>) -> PyResult<ExtendedAsset> {
        match value {
            ExtAssetLike::Raw(raw) => ExtendedAsset::from_bytes(&raw),
            ExtAssetLike::Str(s) => ExtendedAsset::from_str_py(&s),
            ExtAssetLike::Dict(d) => ExtendedAsset::from_dict(d),
            ExtAssetLike::Cls(ext_asset) => Ok(ext_asset)
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

    fn __add__(&self, other: &ExtendedAsset) -> PyResult<ExtendedAsset> {
        let result = self
            .inner
            .quantity
            .try_add(other.inner.quantity)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(NativeExtAsset {
            quantity: result,
            contract: other.inner.contract,
        }.into())
    }

    fn __sub__(&self, other: &ExtendedAsset) -> PyResult<ExtendedAsset> {
        let result = self
            .inner
            .quantity
            .try_sub(other.inner.quantity)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(NativeExtAsset {
            quantity: result,
            contract: other.inner.contract,
        }.into())
    }
}

impl Display for ExtendedAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
