use antminer::AntMinerModel;
use avalonminer::AvalonMinerModel;
use bitaxe::BitAxeModel;
use braiins::BraiinsModel;
use serde::Serialize;
use std::{fmt::Display, str::FromStr};
use whatsminer::WhatsMinerModel;

pub mod antminer;
pub mod avalonminer;
pub mod bitaxe;
pub mod braiins;
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
impl FromStr for BitAxeModel {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ModelParseError)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Copy)]
pub enum MinerModel {
    AntMiner(Option<AntMinerModel>),
    WhatsMiner(Option<WhatsMinerModel>),
    Braiins(Option<BraiinsModel>),
    AvalonMiner(Option<AvalonMinerModel>),
    BitAxe(Option<BitAxeModel>),
}

impl MinerModel {
    pub fn parse_model_str(self: Self, s: &str) -> Self {
        match self {
            MinerModel::AntMiner(_) => MinerModel::AntMiner(AntMinerModel::from_str(s).ok()),
            MinerModel::WhatsMiner(_) => MinerModel::WhatsMiner(WhatsMinerModel::from_str(s).ok()),
            MinerModel::Braiins(_) => MinerModel::Braiins(BraiinsModel::from_str(s).ok()),
            MinerModel::AvalonMiner(_) => {
                MinerModel::AvalonMiner(AvalonMinerModel::from_str(s).ok())
            }
            MinerModel::BitAxe(_) => MinerModel::BitAxe(BitAxeModel::from_str(s).ok()),
        }
    }
}
