use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};

use crate::data::board::{BoardData, ChipData};
use crate::data::device::MinerMake;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::message::{MessageSeverity, MinerMessage};
use crate::data::miner::MinerData;
use crate::data::pool::{PoolData, PoolScheme, PoolURL};
use crate::miners::api::rpc::btminer::BTMinerV3RPC;
use crate::miners::backends::traits::GetMinerData;
use crate::miners::data::{
    DataCollector, DataExtensions, DataExtractor, DataField, DataLocation, get_by_key,
    get_by_pointer,
};
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
    async fn get_data(&self) -> MinerData {
        let mut collector = DataCollector::new(self, &self.rpc);
        let data = collector.collect_all().await;

        // Extract basic string fields
        let mac = data
            .extract::<String>(DataField::Mac)
            .and_then(|s| MacAddr::from_str(&s).ok());

        let hostname = data.extract::<String>(DataField::Hostname);
        let api_version = data.extract::<String>(DataField::ApiVersion);
        let firmware_version = data.extract::<String>(DataField::FirmwareVersion);
        let control_board_version = data.extract::<String>(DataField::ControlBoardVersion);
    }

    fn get_locations(&self, data_field: DataField) -> &'static [DataLocation] {
        const GET_DEVICE_INFO_CMD: &str = "get.device.info";
        const GET_MINER_STATUS_CMD: &str = "get.miner.status";

        match data_field {
            DataField::Mac => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/network/mac"),
                },
            )],
            DataField::ApiVersion => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/api"),
                },
            )],
            DataField::FirmwareVersion => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/fwversion"),
                },
            )],
            DataField::ControlBoardVersion => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/platform"),
                },
            )],
            DataField::SerialNumber => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/miner/miner-sn"),
                },
            )],
            DataField::Hostname => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/network/hostname"),
                },
            )],
            DataField::LightFlashing => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/ledstatus"),
                },
            )],
            DataField::WattageLimit => &[(
                GET_DEVICE_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/miner/power-limit-set"),
                },
            )],
            DataField::Uptime => &[(
                GET_MINER_STATUS_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/power/vout"),
                },
            )],
            _ => &[],
        }
    }
}
