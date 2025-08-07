use super::{MinerFirmware, MinerMake};
use antminer::AntMinerModel;
use avalon::AvalonMinerModel;
use bitaxe::BitaxeModel;
use braiins::BraiinsModel;
use epic::EPicModel;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use whatsminer::WhatsMinerModel;

pub mod antminer;
pub mod avalon;
pub mod bitaxe;
pub mod braiins;
pub mod epic;
pub mod whatsminer;

#[derive(Debug, Clone)]
pub struct ModelParseError;

impl Display for ModelParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse model")
    }
}

impl std::error::Error for ModelParseError {}

impl FromStr for WhatsMinerModel {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ModelParseError)
    }
}
impl FromStr for AntMinerModel {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ModelParseError)
    }
}
impl FromStr for BitaxeModel {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ModelParseError)
    }
}

impl FromStr for BraiinsModel {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ModelParseError)
    }
}

impl FromStr for AvalonMinerModel {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ModelParseError)
    }
}

impl FromStr for EPicModel {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ModelParseError)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinerModel {
    AntMiner(AntMinerModel),
    WhatsMiner(WhatsMinerModel),
    Braiins(BraiinsModel),
    Bitaxe(BitaxeModel),
    Avalon(AvalonMinerModel),
    EPic(EPicModel),
}

impl Display for MinerModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MinerModel::AntMiner(m) => Ok(m.fmt(f)?),
            MinerModel::WhatsMiner(m) => Ok(m.fmt(f)?),
            MinerModel::Braiins(m) => Ok(m.fmt(f)?),
            MinerModel::Bitaxe(m) => Ok(m.fmt(f)?),
            MinerModel::EPic(m) => Ok(m.fmt(f)?),
        }
    }
}

pub(crate) struct MinerModelFactory {
    make: Option<MinerMake>,
    firmware: Option<MinerFirmware>,
}

impl MinerModelFactory {
    pub fn new() -> Self {
        MinerModelFactory {
            make: None,
            firmware: None,
        }
    }

    pub(crate) fn with_make(&mut self, make: MinerMake) -> &Self {
        self.make = Some(make);
        self
    }
    pub(crate) fn with_firmware(&mut self, firmware: MinerFirmware) -> &Self {
        self.firmware = Some(firmware);
        self
    }

    pub(crate) fn parse_model(&self, model_str: &str) -> Option<MinerModel> {
        match self.make {
            Some(MinerMake::AntMiner) => {
                let model = AntMinerModel::from_str(model_str).ok();
                model.map(MinerModel::AntMiner)
            }
            Some(MinerMake::WhatsMiner) => {
                let model = WhatsMinerModel::from_str(model_str).ok();
                model.map(MinerModel::WhatsMiner)
            }
            Some(MinerMake::BitAxe) => {
                let model = BitaxeModel::from_str(model_str).ok();
                model.map(MinerModel::Bitaxe)
            }
            Some(MinerMake::AvalonMiner) => {
                let model = AvalonMinerModel::from_str(model_str).ok();
                model.map(MinerModel::Avalon)
            }
            None => match self.firmware {
                Some(MinerFirmware::BraiinsOS) => {
                    if let Ok(model) = AntMinerModel::from_str(model_str) {
                        return Some(MinerModel::AntMiner(model));
                    }
                    if let Ok(model) = BraiinsModel::from_str(model_str) {
                        return Some(MinerModel::Braiins(model));
                    }
                    None
                }
                Some(MinerFirmware::EPic) => {
                    if let Ok(model) = AntMinerModel::from_str(model_str) {
                        return Some(MinerModel::AntMiner(model));
                    }
                    if let Ok(model) = EPicModel::from_str(model_str) {
                        return Some(MinerModel::EPic(model));
                    }
                    None
                }
                Some(MinerFirmware::LuxOS) => {
                    if let Ok(model) = AntMinerModel::from_str(model_str) {
                        return Some(MinerModel::AntMiner(model));
                    }
                    None
                }
                None => None,
                _ => None,
            },
            _ => None,
        }
    }
}
