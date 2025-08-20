use super::commands::{HTTP_WEB_ROOT, RPC_DEVDETAILS, RPC_VERSION};
use super::model;
use crate::data::device::models::MinerModel;
use crate::data::device::{MinerFirmware, MinerMake};
use crate::miners::commands::MinerCommand;
use semver;
use std::net::IpAddr;

pub(crate) trait DiscoveryCommands {
    fn get_discovery_commands(&self) -> Vec<MinerCommand>;
}
pub(crate) trait ModelSelection {
    async fn get_model(&self, ip: IpAddr) -> Option<MinerModel>;
}

pub(crate) trait VersionSelection {
    async fn get_version(&self, ip: IpAddr) -> Option<semver::Version>;
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
            MinerFirmware::VNish => model::get_model_vnish(ip).await,
            MinerFirmware::EPic => model::get_model_epic(ip).await,
            _ => None,
        }
    }
}
impl VersionSelection for MinerFirmware {
    async fn get_version(&self, ip: IpAddr) -> Option<semver::Version> {
        match self {
            MinerFirmware::VNish => model::get_version_vnish(ip).await,
            MinerFirmware::EPic => model::get_version_epic(ip).await,
            _ => None,
        }
    }
}

impl ModelSelection for MinerMake {
    async fn get_model(&self, ip: IpAddr) -> Option<MinerModel> {
        match self {
            MinerMake::AntMiner => model::get_model_antminer(ip).await,
            MinerMake::WhatsMiner => model::get_model_whatsminer(ip).await,
            MinerMake::BitAxe => model::get_model_bitaxe(ip).await,
            MinerMake::AvalonMiner => model::get_model_avalonminer(ip).await,
            _ => None,
        }
    }
}
impl VersionSelection for MinerMake {
    async fn get_version(&self, ip: IpAddr) -> Option<semver::Version> {
        match self {
            MinerMake::BitAxe => model::get_version_bitaxe(ip).await,
            MinerMake::WhatsMiner => model::get_version_whatsminer(ip).await,
            MinerMake::AntMiner => model::get_version_antminer(ip).await,
            _ => None,
        }
    }
}
