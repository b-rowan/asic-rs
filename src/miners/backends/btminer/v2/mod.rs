pub use rpc::BTMinerRPCAPI;

use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature};
use serde_json::Value;

use crate::data::board::BoardData;
use crate::data::device::MinerMake;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::pool::{PoolData, PoolURL};
use crate::miners::backends::traits::*;
use crate::miners::commands::MinerCommand;
use crate::miners::data::{
    DataCollector, DataExtensions, DataExtractor, DataField, DataLocation, get_by_pointer,
};

mod rpc;

#[derive(Debug)]
pub struct BTMiner2 {
    pub ip: IpAddr,
    pub rpc: BTMinerRPCAPI,
    pub device_info: DeviceInfo,
}

impl BTMiner2 {
    pub fn new(ip: IpAddr, model: MinerModel, firmware: MinerFirmware) -> Self {
        BTMiner2 {
            ip,
            rpc: BTMinerRPCAPI::new(ip, None),
            device_info: DeviceInfo::new(
                MinerMake::WhatsMiner,
                model,
                firmware,
                HashAlgorithm::SHA256,
            ),
        }
    }
}

impl GetDataLocations for BTMiner2 {
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        let get_miner_info_cmd: MinerCommand = MinerCommand::RPC {
            command: "get_miner_info",
            parameters: None,
        };
        let summary_cmd: MinerCommand = MinerCommand::RPC {
            command: "summary",
            parameters: None,
        };
        let devs_cmd: MinerCommand = MinerCommand::RPC {
            command: "devs",
            parameters: None,
        };
        let pools_cmd: MinerCommand = MinerCommand::RPC {
            command: "pools",
            parameters: None,
        };
        let status_cmd: MinerCommand = MinerCommand::RPC {
            command: "status",
            parameters: None,
        };
        let get_version_cmd: MinerCommand = MinerCommand::RPC {
            command: "get_version",
            parameters: None,
        };
        let get_psu_cmd: MinerCommand = MinerCommand::RPC {
            command: "get_psu",
            parameters: None,
        };

        match data_field {
            DataField::Mac => vec![(
                get_miner_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/Msg/mac"),
                    tag: None,
                },
            )],
            DataField::ApiVersion => vec![(
                get_version_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/Msg/api_ver"),
                    tag: None,
                },
            )],
            DataField::FirmwareVersion => vec![(
                get_version_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/Msg/fw_ver"),
                    tag: None,
                },
            )],
            DataField::ControlBoardVersion => vec![(
                get_version_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/Msg/platform"),
                    tag: None,
                },
            )],
            DataField::Hostname => vec![(
                get_miner_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/Msg/hostname"),
                    tag: None,
                },
            )],
            DataField::LightFlashing => vec![(
                get_miner_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/Msg/ledstat"),
                    tag: None,
                },
            )],
            DataField::WattageLimit => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0/Power Limit"),
                    tag: None,
                },
            )],
            DataField::Fans => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0"),
                    tag: None,
                },
            )],
            DataField::PsuFans => vec![(
                get_psu_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/Msg/fan_speed"),
                    tag: None,
                },
            )],
            DataField::Hashboards => vec![(
                devs_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::Pools => vec![(
                pools_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/POOLS"),
                    tag: None,
                },
            )],
            DataField::Uptime => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0/Elapsed"),
                    tag: None,
                },
            )],
            DataField::Wattage => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0/Power"),
                    tag: None,
                },
            )],
            DataField::Hashrate => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0/HS RT"),
                    tag: None,
                },
            )],
            DataField::ExpectedHashrate => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0/Factory GHS"),
                    tag: None,
                },
            )],
            DataField::FluidTemperature => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0/Env Temp"),
                    tag: None,
                },
            )],
            DataField::IsMining => vec![(
                status_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/SUMMARY/0/btmineroff"),
                    tag: None,
                },
            )],
            _ => vec![],
        }
    }
}

impl GetIP for BTMiner2 {
    fn get_ip(&self) -> IpAddr {
        self.ip
    }
}
impl GetDeviceInfo for BTMiner2 {
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info
    }
}

impl CollectData for BTMiner2 {
    fn get_collector(&self) -> DataCollector<'_> {
        DataCollector::new(self, &self.rpc)
    }
}

impl GetMAC for BTMiner2 {
    fn parse_mac(&self, data: &HashMap<DataField, Value>) -> Option<MacAddr> {
        data.extract::<String>(DataField::Mac)
            .and_then(|s| MacAddr::from_str(&s).ok())
    }
}

impl GetSerialNumber for BTMiner2 {}
impl GetHostname for BTMiner2 {
    fn parse_hostname(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::Hostname)
    }
}
impl GetApiVersion for BTMiner2 {
    fn parse_api_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::ApiVersion)
    }
}
impl GetFirmwareVersion for BTMiner2 {
    fn parse_firmware_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::FirmwareVersion)
    }
}
impl GetControlBoardVersion for BTMiner2 {
    fn parse_control_board_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::ControlBoardVersion)
    }
}
impl GetHashboards for BTMiner2 {
    fn parse_hashboards(&self, data: &HashMap<DataField, Value>) -> Vec<BoardData> {
        let mut hashboards: Vec<BoardData> = Vec::new();
        let board_count = self.device_info.hardware.boards.unwrap_or(3);
        for idx in 0..board_count {
            let hashrate = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/MHS av", idx)))
                .and_then(|val| val.as_f64())
                .map(|f| {
                    HashRate {
                        value: f,
                        unit: HashRateUnit::MegaHash,
                        algo: String::from("SHA256"),
                    }
                    .as_unit(HashRateUnit::TeraHash)
                });
            let expected_hashrate = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/Factory GHS", idx)))
                .and_then(|val| val.as_f64())
                .map(|f| {
                    HashRate {
                        value: f,
                        unit: HashRateUnit::GigaHash,
                        algo: String::from("SHA256"),
                    }
                    .as_unit(HashRateUnit::TeraHash)
                });
            let board_temperature = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/Temperature", idx)))
                .and_then(|val| val.as_f64())
                .map(Temperature::from_celsius);
            let intake_temperature = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/Chip Temp Min", idx)))
                .and_then(|val| val.as_f64())
                .map(Temperature::from_celsius);
            let outlet_temperature = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/Chip Temp Max", idx)))
                .and_then(|val| val.as_f64())
                .map(Temperature::from_celsius);
            let serial_number = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/PCB SN", idx)))
                .and_then(|val| val.as_str())
                .map(String::from);
            let working_chips = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/Effective Chips", idx)))
                .and_then(|val| val.as_u64())
                .map(|u| u as u16);
            let frequency = data
                .get(&DataField::Hashboards)
                .and_then(|val| val.pointer(&format!("/DEVS/{}/Frequency", idx)))
                .and_then(|val| val.as_f64())
                .map(Frequency::from_megahertz);

            let active = Some(hashrate.clone().map(|h| h.value).unwrap_or(0f64) > 0f64);
            hashboards.push(BoardData {
                hashrate,
                position: idx,
                expected_hashrate,
                board_temperature,
                intake_temperature,
                outlet_temperature,
                expected_chips: self.device_info.hardware.chips,
                working_chips,
                serial_number,
                chips: vec![],
                voltage: None, // TODO
                frequency,
                tuned: Some(true),
                active,
            });
        }
        hashboards
    }
}
impl GetHashrate for BTMiner2 {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::Hashrate, |f| {
            HashRate {
                value: f,
                unit: HashRateUnit::MegaHash,
                algo: String::from("SHA256"),
            }
            .as_unit(HashRateUnit::TeraHash)
        })
    }
}
impl GetExpectedHashrate for BTMiner2 {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::ExpectedHashrate, |f| {
            HashRate {
                value: f,
                unit: HashRateUnit::GigaHash,
                algo: String::from("SHA256"),
            }
            .as_unit(HashRateUnit::TeraHash)
        })
    }
}
impl GetFans for BTMiner2 {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        let mut fans: Vec<FanData> = Vec::new();
        for (idx, direction) in ["In", "Out"].iter().enumerate() {
            let fan = data.extract_nested_map::<f64, _>(
                DataField::Fans,
                &format!("Fan Speed {}", direction),
                |rpm| FanData {
                    position: idx as i16,
                    rpm: Some(AngularVelocity::from_rpm(rpm)),
                },
            );
            if fan.is_some() {
                fans.push(fan.unwrap());
            }
        }
        fans
    }
}
impl GetPsuFans for BTMiner2 {
    fn parse_psu_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        let mut psu_fans: Vec<FanData> = Vec::new();

        let psu_fan = data.extract_map::<String, _>(DataField::PsuFans, |rpm| FanData {
            position: 0i16,
            rpm: Some(AngularVelocity::from_rpm(rpm.parse().unwrap())),
        });
        if psu_fan.is_some() {
            psu_fans.push(psu_fan.unwrap());
        }
        psu_fans
    }
}
impl GetFluidTemperature for BTMiner2 {
    fn parse_fluid_temperature(&self, data: &HashMap<DataField, Value>) -> Option<Temperature> {
        data.extract_map::<f64, _>(DataField::FluidTemperature, Temperature::from_celsius)
    }
}
impl GetWattage for BTMiner2 {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<f64, _>(DataField::Wattage, Power::from_watts)
    }
}
impl GetWattageLimit for BTMiner2 {
    fn parse_wattage_limit(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<f64, _>(DataField::WattageLimit, Power::from_watts)
    }
}
impl GetLightFlashing for BTMiner2 {
    fn parse_light_flashing(&self, data: &HashMap<DataField, Value>) -> Option<bool> {
        data.extract_map::<String, _>(DataField::LightFlashing, |l| l != "auto")
    }
}
impl GetMessages for BTMiner2 {} // TODO
impl GetUptime for BTMiner2 {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.extract_map::<u64, _>(DataField::Uptime, Duration::from_secs)
    }
}
impl GetIsMining for BTMiner2 {
    fn parse_is_mining(&self, data: &HashMap<DataField, Value>) -> bool {
        data.extract_map::<String, _>(DataField::IsMining, |l| l != "false")
            .unwrap_or(true)
    }
}
impl GetPools for BTMiner2 {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolData> {
        let mut pools: Vec<PoolData> = Vec::new();
        let pools_raw = data.get(&DataField::Pools);
        if pools_raw.is_some() {
            let pools_response = pools_raw.unwrap();
            for (idx, _) in pools_response
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .enumerate()
            {
                let user = data
                    .get(&DataField::Pools)
                    .and_then(|val| val.pointer(&format!("/{}/User", idx)))
                    .map(|val| String::from(val.as_str().unwrap_or("")));

                let alive = data
                    .get(&DataField::Pools)
                    .and_then(|val| val.pointer(&format!("/{}/Status", idx)))
                    .map(|val| val.as_str())
                    .map(|val| val == Some("Alive"));

                let active = data
                    .get(&DataField::Pools)
                    .and_then(|val| val.pointer(&format!("/{}/Stratum Active", idx)))
                    .and_then(|val| val.as_bool());

                let url = data
                    .get(&DataField::Pools)
                    .and_then(|val| val.pointer(&format!("/{}/URL", idx)))
                    .map(|val| PoolURL::from(String::from(val.as_str().unwrap_or(""))));

                let accepted_shares = data
                    .get(&DataField::Pools)
                    .and_then(|val| val.pointer(&format!("/{}/Accepted", idx)))
                    .and_then(|val| val.as_u64());

                let rejected_shares = data
                    .get(&DataField::Pools)
                    .and_then(|val| val.pointer(&format!("/{}/Rejected", idx)))
                    .and_then(|val| val.as_u64());

                pools.push(PoolData {
                    position: Some(idx as u16),
                    url,
                    accepted_shares,
                    rejected_shares,
                    active,
                    alive,
                    user,
                });
            }
        }
        pools
    }
}
