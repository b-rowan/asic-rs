use serde::{Deserialize, Serialize};
use strum::Display;

pub mod models;
pub use models::MinerModel;
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub make: MinerMake,
    pub model: MinerModel,
    pub hardware: MinerHardware,
    pub firmware: MinerFirmware,
    pub algo: HashAlgorithm,
}

impl DeviceInfo {
    pub(crate) fn new(
        make: MinerMake,
        model: MinerModel,
        firmware: MinerFirmware,
        algo: HashAlgorithm,
    ) -> Self {
        Self {
            make,
            hardware: MinerHardware::from(&model),
            model,
            firmware,
            algo,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct MinerHardware {
    pub chips: Option<u16>,
    pub fans: Option<u8>,
    pub boards: Option<u8>,
}

impl From<&MinerModel> for MinerHardware {
    fn from(model: &MinerModel) -> Self {
        match model {
            MinerModel::AntMiner(model_name) => Self::from(model_name),
            MinerModel::WhatsMiner(model_name) => Self::from(model_name),
            MinerModel::Braiins(model_name) => Self::from(model_name),
            MinerModel::Bitaxe(model_name) => Self::from(model_name),
            MinerModel::EPic(model_name) => Self::from(model_name),
        }
    }
}
