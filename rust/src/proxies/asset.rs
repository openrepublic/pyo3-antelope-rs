use crate::impl_packable_py;
use crate::proxies::name::Name;
use crate::proxies::sym::Symbol;
use antelope::chain::asset::{
    Asset as NativeAsset, ExtendedAsset as NativeExtAsset, Symbol as NativeSymbol,
};
use antelope::serializer::Packer;
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rust_decimal::Decimal;
use std::fmt::Display;
use std::str::FromStr;

use super::name::NameLike;
use super::sym::SymLike;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Asset {
    pub inner: NativeAsset,
}

#[derive(FromPyObject)]
pub enum AssetLike<'py> {
    Str(String),
    Ints(i64, u8, String),
    Decimal(Decimal, u8, String),
    Dict(Bound<'py, PyDict>),
    Cls(Asset)
}

impl_packable_py! {
    impl Asset(NativeAsset) {
        #[new]
        fn new<'py>(amount: i64, sym: SymLike) -> PyResult<Self> {
            let sym = Symbol::try_from(sym)?;
            let inner = NativeAsset::try_from((amount, sym.inner))
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(Asset { inner })
        }

        #[staticmethod]
        pub fn try_from<'py>(value: AssetLike<'py>) -> PyResult<Asset> {
            match value {
                AssetLike::Str(s) => Asset::from_str(&s),
                AssetLike::Ints(amount, precision, sym) => Asset::from_ints(amount, precision, &sym),
                AssetLike::Decimal(d, precision, sym) => Asset::from_decimal(d, precision, &sym),
                AssetLike::Dict(d) => Asset::from_dict(d),
                AssetLike::Cls(asset) => Ok(asset)
            }
        }

        #[staticmethod]
        fn from_str(s: &str) -> PyResult<Self> {
            Ok(Asset { inner: NativeAsset::from_str(s)
                .map_err(|e| PyValueError::new_err(e.to_string()))? })
        }

        #[staticmethod]
        fn from_ints(amount: i64, precision: u8, sym: &str) -> PyResult<Self> {
            Ok(Asset {
                inner: NativeAsset::try_from((
                    amount,
                    NativeSymbol::try_from((sym, precision))
                        .map_err(|e| PyValueError::new_err(e.to_string()))?
                )).map_err(|e| PyValueError::new_err(e.to_string()))?,
            })
        }

        #[staticmethod]
        fn from_decimal(d: Decimal, precision: u8, sym: &str) -> PyResult<Self> {
            let d_str = d.to_string();
            let dot_idx = d_str.find('.')
                .unwrap_or(Err(PyValueError::new_err("Could not find decimal point"))?);

            let num_str = d_str[..dot_idx + 1 + precision as usize].to_string();
            Ok(Asset::from_str(&format!("{} {}", num_str, sym))?)
        }

        #[staticmethod]
        fn from_dict<'py>(d: Bound<'py, PyDict>) -> PyResult<Self> {
            let py_amount = d.get_item("amount")?
                .ok_or(PyKeyError::new_err("Expected asset dict to have amount key"))?
                .extract()?;

            let py_symbol = d.get_item("symbol")?
                .ok_or(PyKeyError::new_err("Expected asset dict to have amount key"))?
                .extract()?;

            Asset::new(py_amount, py_symbol)
        }

        fn to_decimal(&self) -> Decimal {
            let mut str_amount = format!("{:0>width$}", self.amount(), width = (self.symbol().precision() + 1) as usize);

            if self.symbol().precision() > 0 {
                let len = str_amount.len();
                str_amount.insert(len - self.symbol().precision() as usize, '.');
            }

            Decimal::from_str(&str_amount).unwrap_or(Decimal::ZERO)
        }

        /// Return the i64 amount
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
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.to_string())
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ExtendedAsset {
    pub quantity: Asset,
    pub contract: Name,
}

impl From<&ExtendedAsset> for NativeExtAsset {
    fn from(value: &ExtendedAsset) -> Self {
        NativeExtAsset {
            quantity: value.quantity.inner,
            contract: value.contract.inner,
        }
    }
}

#[pymethods]
impl ExtendedAsset {
    #[staticmethod]
    fn from_dict<'py>(d: Bound<'py, PyDict>) -> PyResult<Self> {
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

        Ok(ExtendedAsset{ quantity, contract })
    }

    #[staticmethod]
    fn from_str(s: &str) -> PyResult<Self> {
        let ext = NativeExtAsset::from_str(s).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(ExtendedAsset {
            quantity: Asset {
                inner: ext.quantity,
            },
            contract: Name {
                inner: ext.contract,
            },
        })
    }

    fn __str__(&self) -> String {
        Into::<NativeExtAsset>::into(self).to_string()
    }

    fn __add__(&self, other: &ExtendedAsset) -> PyResult<ExtendedAsset> {
        let result = self
            .quantity
            .inner
            .try_add(other.quantity.inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(ExtendedAsset {
            quantity: Asset { inner: result },
            contract: other.contract.clone(),
        })
    }

    fn __sub__(&self, other: &ExtendedAsset) -> PyResult<ExtendedAsset> {
        let result = self
            .quantity
            .inner
            .try_sub(other.quantity.inner)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(ExtendedAsset {
            quantity: Asset { inner: result },
            contract: other.contract.clone(),
        })
    }
}

impl Display for ExtendedAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<NativeExtAsset>::into(self).to_string())
    }
}
