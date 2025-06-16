use serde::{Deserialize, Serialize};

pub mod models;
pub use models::MinerModel;
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MinerFirmware {
    #[serde(rename = "Stock")]
    Stock,
    #[serde(rename = "BraiinsOS")]
    BraiinsOS,
    #[serde(rename = "VNish")]
    VNish,
    #[serde(rename = "ePIC")]
    EPic,
    #[serde(rename = "HiveOS")]
    HiveOS,
    #[serde(rename = "LuxOS")]
    LuxOS,
    #[serde(rename = "Marathon")]
    Marathon,
    #[serde(rename = "MSKMiner")]
    MSKMiner,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MinerMake {
    #[serde(rename = "AntMiner")]
    AntMiner,
    #[serde(rename = "WhatsMiner")]
    WhatsMiner,
    #[serde(rename = "AvalonMiner")]
    AvalonMiner,
    #[serde(rename = "ePIC")]
    EPic,
    #[serde(rename = "Braiins")]
    Braiins,
    #[serde(rename = "BitAxe")]
    BitAxe,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum HashAlgorithm {
    #[serde(rename = "SHA256")]
    SHA256,
    #[serde(rename = "Scrypt")]
    Scrypt,
    #[serde(rename = "X11")]
    X11,
    #[serde(rename = "Blake2S256")]
    Blake2S256,
    #[serde(rename = "Kadena")]
    Kadena,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub model: MinerModel,
    pub hardware: MinerHardware,
    pub firmware: MinerFirmware,
    pub algo: HashAlgorithm,
}

impl DeviceInfo {
    pub(crate) fn new(model: MinerModel, firmware: MinerFirmware, algo: HashAlgorithm) -> Self {
        Self {
            hardware: MinerHardware::from(&model),
            model,
            firmware,
            algo,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinerHardware {
    pub chips: Option<u16>,
    pub fans: Option<u8>,
    pub boards: Option<u8>,
}

impl From<&MinerModel> for MinerHardware {
    fn from(model: &MinerModel) -> Self {
        match model {
            MinerModel::AntMiner(Some(model_name)) => Self::from(model_name),
            MinerModel::WhatsMiner(Some(model_name)) => Self::from(model_name),
            MinerModel::Braiins(Some(model_name)) => Self::from(model_name),
            _ => Self {
                chips: None,
                fans: None,
                boards: None,
            },
        }
    }
}
