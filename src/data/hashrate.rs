use measurements::Power;
use std::ops::Div;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct HashRate {
    pub value: f64,
    pub unit: HashRateUnit,
    pub algo: String,
}

impl Div<HashRate> for Power {
    type Output = f64;

    fn div(self, hash_rate: HashRate) -> Self::Output {
        self.as_watts() / hash_rate.value
    }
}
