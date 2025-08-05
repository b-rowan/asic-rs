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
