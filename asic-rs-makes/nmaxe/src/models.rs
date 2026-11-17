use std::str::FromStr;

use asic_rs_core::{
    data::device::HashAlgorithm, errors::ModelSelectionError, traits::model::MinerModel,
};
use asic_rs_macros::ModelAlgorithm;
use serde::{Deserialize, Serialize};
use strum::Display;
use ts_rs::TS;

#[derive(
    Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Display, ModelAlgorithm, TS,
)]
pub enum NMAxeModel {
    #[algorithm(HashAlgorithm::SHA256)]
    NMAxe,
    #[algorithm(HashAlgorithm::SHA256)]
    NMAxeGamma,
    // Rev6.1 retains the base model's BM1370 ASICs and chip/fan counts.
    #[serde(rename = "NMQAxe++", alias = "NMQAxe++Rev6.1")]
    #[strum(to_string = "NMQAxe++")]
    #[algorithm(HashAlgorithm::SHA256)]
    NMQAxePlusPlus,
    // Rev8.1 changes the ASIC generation from BM1370 to BM1373.
    #[serde(rename = "NMQAxe++Rev8.1")]
    #[strum(to_string = "NMQAxe++Rev8.1")]
    #[algorithm(HashAlgorithm::SHA256)]
    NMQAxePlusPlusRev81,
    #[serde(rename = "NMQAxe++Nexus")]
    #[strum(to_string = "NMQAxe++Nexus")]
    #[algorithm(HashAlgorithm::SHA256)]
    NMQAxePlusPlusNexus,
    #[strum(to_string = "{0}")]
    #[algorithm(HashAlgorithm::SHA256)]
    Unknown(String),
}

impl FromStr for NMAxeModel {
    type Err = ModelSelectionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .or_else(|_| Ok(Self::Unknown(s.to_string())))
    }
}

impl MinerModel for NMAxeModel {
    fn make_name(&self) -> String {
        "NMAxe".to_string()
    }
    fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}
