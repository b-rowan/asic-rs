use crate::data::board::BoardData;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerMake, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::message::MinerMessage;
use crate::data::miner::MinerData;
use crate::data::pool::{PoolData, PoolURL};
use crate::miners::api::rpc::errors::RPCError;
use crate::miners::api::rpc::{btminer::BTMinerV3RPC, traits::SendRPCCommand};
use crate::miners::backends::traits::GetMinerData;
use crate::miners::data::{DataExtractor, DataField, DataLocation, get_by_key, get_by_pointer};
use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct BTMiner3 {
    pub ip: IpAddr,
    pub rpc: BTMinerV3RPC,
    pub device_info: DeviceInfo,
}

impl BTMiner3 {
    pub fn new(ip: IpAddr, model: MinerModel, firmware: MinerFirmware) -> Self {
        BTMiner3 {
            ip,
            rpc: BTMinerV3RPC::new(ip, None),
            device_info: DeviceInfo::new(
                MinerMake::WhatsMiner,
                model,
                firmware,
                HashAlgorithm::SHA256,
            ),
        }
    }
}

#[async_trait]
impl GetMinerData for BTMiner3 {
    async fn get_data(&self) -> MinerData {}

    fn get_locations(&self, data_field: DataField) -> &'static [DataLocation] {
        const GET_DEVICE_INFO_CMD: &str = "get.device.info";
        const GET_MINER_STATUS_CMD: &str = "get.miner.status";

        match data_field {
            _ => &[],
        }
    }
}
