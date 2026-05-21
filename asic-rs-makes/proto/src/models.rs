use std::{fmt::Display, str::FromStr};

use asic_rs_core::{errors::ModelSelectionError, traits::model::MinerModel};
use serde::{Deserialize, Serialize};

use crate::hardware::ProtoHardwareConfig;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ProtoModel {
    name: String,
    hardware: ProtoHardwareConfig,
}

impl ProtoModel {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hardware: ProtoHardwareConfig::default(),
        }
    }

    pub fn with_hardware(name: impl Into<String>, hardware: ProtoHardwareConfig) -> Self {
        Self {
            name: name.into(),
            hardware,
        }
    }

    pub fn hardware(&self) -> &ProtoHardwareConfig {
        &self.hardware
    }

    pub fn into_hardware(self) -> ProtoHardwareConfig {
        self.hardware
    }
}

impl Display for ProtoModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromStr for ProtoModel {
    type Err = ModelSelectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.trim()))
    }
}

impl MinerModel for ProtoModel {
    fn make_name(&self) -> String {
        "Proto".to_string()
    }
}

#[cfg(test)]
mod tests {
    use asic_rs_core::data::device::MinerHardware;

    use super::*;
    use crate::hardware::{ProtoHashboardConfig, ProtoHashboardKind};

    #[test]
    fn heterogeneous_hardware_does_not_claim_uniform_chip_count() {
        let hardware = ProtoHardwareConfig::new(
            vec![
                ProtoHashboardConfig::new(0, ProtoHashboardKind::B3a, Some(100)),
                ProtoHashboardConfig::new(1, ProtoHashboardKind::B4, Some(120)),
            ],
            Some(4),
        );

        let model = ProtoModel::with_hardware("Rig", hardware);
        let summary = MinerHardware::from(model);

        assert_eq!(summary.boards, Some(2));
        assert_eq!(summary.fans, Some(4));
        assert_eq!(summary.chips, None);
    }
}
