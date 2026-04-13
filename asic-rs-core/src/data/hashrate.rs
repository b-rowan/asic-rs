use std::{
    fmt::{Display, Formatter},
    ops::Div,
    str::FromStr,
};

use asic_rs_derive::PydanticCompat;
use measurements::Power;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "python", pyclass(from_py_object, str, module = "asic_rs"))]
#[cfg_attr(feature = "python", derive(PydanticCompat))]
#[cfg_attr(
    feature = "python",
    pydantic(validate = "from_str", serialize = "to_string")
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HashRateUnit {
    Hash,
    KiloHash,
    MegaHash,
    GigaHash,
    #[default]
    TeraHash,
    PetaHash,
    ExaHash,
    ZettaHash,
    YottaHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashRateUnitParseError {
    input: String,
}

impl Display for HashRateUnitParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown hash rate unit: {}", self.input)
    }
}

impl std::error::Error for HashRateUnitParseError {}

impl HashRateUnit {
    fn to_multiplier(&self) -> f64 {
        match self {
            HashRateUnit::Hash => 1e0,
            HashRateUnit::KiloHash => 1e3,
            HashRateUnit::MegaHash => 1e6,
            HashRateUnit::GigaHash => 1e9,
            HashRateUnit::TeraHash => 1e12,
            HashRateUnit::PetaHash => 1e15,
            HashRateUnit::ExaHash => 1e18,
            HashRateUnit::ZettaHash => 1e21,
            HashRateUnit::YottaHash => 1e24,
        }
    }
}

impl FromStr for HashRateUnit {
    type Err = HashRateUnitParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_uppercase().replace([' ', '_'], "");

        match normalized.as_str() {
            "HASH" | "H" | "HS" | "H/S" => Ok(HashRateUnit::Hash),
            "KILOHASH" | "KH" | "KHS" | "KH/S" => Ok(HashRateUnit::KiloHash),
            "MEGAHASH" | "MH" | "MHS" | "MH/S" => Ok(HashRateUnit::MegaHash),
            "GIGAHASH" | "GH" | "GHS" | "GH/S" => Ok(HashRateUnit::GigaHash),
            "TERAHASH" | "TH" | "THS" | "TH/S" => Ok(HashRateUnit::TeraHash),
            "PETAHASH" | "PH" | "PHS" | "PH/S" => Ok(HashRateUnit::PetaHash),
            "EXAHASH" | "EH" | "EHS" | "EH/S" => Ok(HashRateUnit::ExaHash),
            "ZETTAHASH" | "ZH" | "ZHS" | "ZH/S" => Ok(HashRateUnit::ZettaHash),
            "YOTTAHASH" | "YH" | "YHS" | "YH/S" => Ok(HashRateUnit::YottaHash),
            _ => Err(HashRateUnitParseError {
                input: s.to_string(),
            }),
        }
    }
}

impl Display for HashRateUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HashRateUnit::Hash => write!(f, "H/s"),
            HashRateUnit::KiloHash => write!(f, "KH/s"),
            HashRateUnit::MegaHash => write!(f, "MH/s"),
            HashRateUnit::GigaHash => write!(f, "GH/s"),
            HashRateUnit::TeraHash => write!(f, "TH/s"),
            HashRateUnit::PetaHash => write!(f, "PH/s"),
            HashRateUnit::ExaHash => write!(f, "EH/s"),
            HashRateUnit::ZettaHash => write!(f, "ZH/s"),
            HashRateUnit::YottaHash => write!(f, "YH/s"),
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl HashRateUnit {
    fn __repr__(&self) -> String {
        self.to_string()
    }
}

#[cfg_attr(feature = "python", derive(PydanticCompat))]
#[cfg_attr(
    feature = "python",
    pyclass(from_py_object, get_all, module = "asic_rs")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashRate {
    /// The current amount of hashes being computed
    pub value: f64,
    /// The unit of the hashes in value — accepts string or `HashRateUnit` when
    /// validating; serialises to the display string (e.g. `"TH/s"`).
    #[cfg_attr(
        feature = "python",
        pydantic(validate_with = "from_str", serialize_with = "to_string")
    )]
    pub unit: HashRateUnit,
    /// The algorithm of the computed hashes
    pub algo: String,
}

impl HashRate {
    pub fn as_unit(self, unit: HashRateUnit) -> Self {
        let base = self.value * self.unit.to_multiplier();

        Self {
            value: base / unit.clone().to_multiplier(),
            unit,
            algo: self.algo,
        }
    }
}

impl Display for HashRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let precision = f.precision();

        match precision {
            Some(precision) => write!(f, "{:.*} {}", precision, self.value, self.unit),
            None => write!(f, "{} {}", self.value, self.unit),
        }
    }
}

impl PartialEq for HashRate {
    fn eq(&self, other: &Self) -> bool {
        other.clone().as_unit(self.unit.clone()).value == self.value
    }
}

impl Eq for HashRate {}

impl Div<HashRate> for Power {
    type Output = f64;

    fn div(self, hash_rate: HashRate) -> Self::Output {
        self.as_watts() / hash_rate.value
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl HashRate {
    fn __repr__(&self) -> String {
        self.to_string()
    }

    fn __float__(&self) -> f64 {
        self.value
    }

    fn __format__<'py>(&self, py: Python<'py>, format_spec: &str) -> PyResult<String> {
        let py_value = self.value.into_pyobject(py)?;
        let formatted: String = py_value
            .call_method1("__format__", (format_spec,))?
            .extract()?;
        Ok(format!("{} {}", formatted, self.unit))
    }

    /// Convert this `HashRate` to the given unit, returning a new instance.
    fn into_unit(&self, unit: HashRateUnit) -> HashRate {
        self.clone().as_unit(unit)
    }
}
