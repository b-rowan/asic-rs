use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MinerFirmware {
    Stock,
    BraiinsOS,
    VNish,
    EPic,
    HiveOn,
    LuxOS,
    Marathon,
    MSKMiner,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MinerMake {
    AntMiner,
    WhatsMiner,
    AvalonMiner,
    EPic,
    Braiins,
    BitAxe,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Algorithm {
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
    #[serde(rename = "Unknown")]
    Unknown,
}

// Keep your existing AntminerModel enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntminerModel {
    #[serde(rename = "ANTMINER D3")]
    D3,
    #[serde(rename = "ANTMINER HS3")]
    Hs3,
    #[serde(rename = "ANTMINER L3+")]
    L3Plus,
    #[serde(rename = "ANTMINER KA3")]
    Ka3,
    #[serde(rename = "ANTMINER KS3")]
    Ks3,
    // Add all other models as needed
    #[serde(other)]
    Unknown,
}

// Add a WhatsmimerModel enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhatsmimerModel {
    #[serde(rename = "WHATSMINER M30S")]
    M30S,
    #[serde(rename = "WHATSMINER M30S+")]
    M30SPlus,
    #[serde(rename = "WHATSMINER M31S")]
    M31S,
    #[serde(rename = "WHATSMINER M31S+")]
    M31SPlus,
    // Add other Whatsminer models
    #[serde(other)]
    Unknown,
}

// Create a unified MinerModel enum that can be either type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinerModel {
    Antminer(AntminerModel),
    Whatsminer(WhatsmimerModel),
    Unknown,
}

// Model spec information
#[derive(Debug, Clone, Copy)]
pub struct ModelSpec {
    pub hashrate: f64, // TH/s
    pub power: u32,    // Watts
    pub algorithm: Algorithm,
}

// Implement methods for AntminerModel
impl AntminerModel {
    pub fn from_string(model_str: &str) -> Self {
        serde_json::from_value(serde_json::Value::String(model_str.to_string()))
            .unwrap_or(AntminerModel::Unknown)
    }

    pub fn spec(&self) -> ModelSpec {
        match self {
            Self::D3 => ModelSpec {
                hashrate: 19.3,
                power: 1600,
                algorithm: Algorithm::X11,
            },
            Self::Hs3 => ModelSpec {
                hashrate: 0.85,
                power: 1350,
                algorithm: Algorithm::Blake2S256,
            },
            Self::L3Plus => ModelSpec {
                hashrate: 0.504,
                power: 800,
                algorithm: Algorithm::Scrypt,
            },
            Self::Ka3 => ModelSpec {
                hashrate: 166.0,
                power: 3154,
                algorithm: Algorithm::Kadena,
            },
            _ => ModelSpec {
                hashrate: 0.0,
                power: 0,
                algorithm: Algorithm::Unknown,
            },
        }
    }
}

// Implement methods for WhatsmimerModel
impl WhatsmimerModel {
    pub fn from_string(model_str: &str) -> Self {
        serde_json::from_value(serde_json::Value::String(model_str.to_string()))
            .unwrap_or(WhatsmimerModel::Unknown)
    }

    pub fn spec(&self) -> ModelSpec {
        match self {
            Self::M30S => ModelSpec {
                hashrate: 88.0,
                power: 3344,
                algorithm: Algorithm::SHA256,
            },
            Self::M30SPlus => ModelSpec {
                hashrate: 100.0,
                power: 3400,
                algorithm: Algorithm::SHA256,
            },
            // Add specs for other models
            _ => ModelSpec {
                hashrate: 0.0,
                power: 0,
                algorithm: Algorithm::Unknown,
            },
        }
    }
}

// Implement methods for the unified MinerModel
impl MinerModel {
    pub fn from_string(make: MinerMake, model_str: &str) -> Self {
        match make {
            MinerMake::AntMiner => MinerModel::Antminer(AntminerModel::from_string(model_str)),
            MinerMake::WhatsMiner => {
                MinerModel::Whatsminer(WhatsmimerModel::from_string(model_str))
            }
            _ => MinerModel::Unknown,
        }
    }

    pub fn spec(&self) -> ModelSpec {
        match self {
            MinerModel::Antminer(model) => model.spec(),
            MinerModel::Whatsminer(model) => model.spec(),
            MinerModel::Unknown => ModelSpec {
                hashrate: 0.0,
                power: 0,
                algorithm: Algorithm::Unknown,
            },
        }
    }
}

// Update DeviceInfo to use the unified MinerModel
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub make: MinerMake,
    pub model: String,
    pub miner_model: Option<MinerModel>,
    pub firmware: MinerFirmware,
}

impl DeviceInfo {
    pub fn new(make: MinerMake, model: &str, firmware: MinerFirmware) -> Self {
        DeviceInfo {
            make,
            model: model.to_string(),
            miner_model: Some(MinerModel::from_string(make, model)),
            firmware,
        }
    }

    // Helper methods to get model-specific information
    pub fn hashrate(&self) -> Option<f64> {
        self.miner_model.map(|model| model.spec().hashrate)
    }

    pub fn power_consumption(&self) -> Option<u32> {
        self.miner_model.map(|model| model.spec().power)
    }

    pub fn algorithm(&self) -> Option<Algorithm> {
        self.miner_model.map(|model| model.spec().algorithm)
    }
}
