use measurements::Power;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    ops::Div,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashRateUnit {
    Hash,
    KiloHash,
    MegaHash,
    GigaHash,
    TeraHash,
    PetaHash,
    ExaHash,
    ZettaHash,
    YottaHash,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HashRate {
    /// The current amount of hashes being computed
    pub value: f64,
    /// The unit of the hashes in value
    pub unit: HashRateUnit,
    /// The algorithm of the computed hashes
    pub algo: String,
}

impl HashRate {
    pub fn as_unit(self, unit: HashRateUnit) -> Self {
        let base = self.value * self.unit.to_multiplier(); // Convert to base unit (e.g., bytes)

        Self {
            value: base / unit.clone().to_multiplier(),
            unit,
            algo: self.algo,
        }
    }
}

impl Display for HashRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.unit)
    }
}
impl Div<HashRate> for Power {
    type Output = f64;

    fn div(self, hash_rate: HashRate) -> Self::Output {
        self.as_watts() / hash_rate.value
    }
}
