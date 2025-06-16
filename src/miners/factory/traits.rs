use super::commands::{HTTP_WEB_ROOT, RPC_DEVDETAILS, RPC_VERSION};
use super::model;
use crate::data::device::models::MinerModel;
use crate::data::device::{MinerFirmware, MinerMake};
use crate::miners::commands::MinerCommand;
use std::net::IpAddr;

pub(crate) trait DiscoveryCommands {
    fn get_discovery_commands(&self) -> Vec<MinerCommand>;
}
pub(crate) trait ModelSelection {
    async fn get_model(&self, ip: IpAddr) -> Option<MinerModel>;
}

impl DiscoveryCommands for MinerMake {
    fn get_discovery_commands(&self) -> Vec<MinerCommand> {
        match self {
            MinerMake::AntMiner => vec![RPC_VERSION, HTTP_WEB_ROOT],
            MinerMake::WhatsMiner => vec![RPC_DEVDETAILS, HTTP_WEB_ROOT],
            MinerMake::AvalonMiner => vec![RPC_VERSION, HTTP_WEB_ROOT],
            MinerMake::EPic => vec![HTTP_WEB_ROOT],
            MinerMake::Braiins => vec![RPC_VERSION, HTTP_WEB_ROOT],
            MinerMake::BitAxe => vec![HTTP_WEB_ROOT],
        }
    }
}
impl DiscoveryCommands for MinerFirmware {
    fn get_discovery_commands(&self) -> Vec<MinerCommand> {
        match self {
            MinerFirmware::Stock => vec![], // stock firmware needs miner make
            MinerFirmware::BraiinsOS => vec![RPC_VERSION, HTTP_WEB_ROOT],
            MinerFirmware::VNish => vec![HTTP_WEB_ROOT, RPC_VERSION],
            MinerFirmware::EPic => vec![HTTP_WEB_ROOT],
            MinerFirmware::HiveOS => vec![],
            MinerFirmware::LuxOS => vec![HTTP_WEB_ROOT, RPC_VERSION],
            MinerFirmware::Marathon => vec![],
            MinerFirmware::MSKMiner => vec![],
        }
    }
}
impl ModelSelection for MinerFirmware {
    async fn get_model(&self, ip: IpAddr) -> Option<MinerModel> {
        match self {
            MinerFirmware::LuxOS => model::get_model_luxos(ip).await,
            MinerFirmware::BraiinsOS => model::get_model_braiins_os(ip).await,
            _ => None,
        }
    }
}
impl ModelSelection for MinerModel {
    async fn get_model(&self, ip: IpAddr) -> Option<MinerModel> {
        match self {
            MinerModel::AntMiner(_) => model::get_model_antminer(ip).await,
            MinerModel::WhatsMiner(_) => model::get_model_whatsminer(ip).await,
            _ => None,
        }
    }
}
