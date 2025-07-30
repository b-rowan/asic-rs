use std::collections::HashMap;
use std::net::IpAddr;
use std::ops::Deref;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};
use serde_json::{Value, json};

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
use crate::miners::commands::MinerCommand;
use crate::miners::data::{
    DataCollector, DataExtensions, DataExtractor, DataField, DataLocation, get_by_key,
    get_by_pointer,
};

#[derive(Debug)]
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

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_secs();

        // Extract basic string fields
        let mac = data
            .extract::<String>(DataField::Mac)
            .and_then(|s| MacAddr::from_str(&s).ok());

        let hostname = data.extract::<String>(DataField::Hostname);
        let api_version = data.extract::<String>(DataField::ApiVersion);
        let firmware_version = data.extract::<String>(DataField::FirmwareVersion);
        let control_board_version = data.extract::<String>(DataField::ControlBoardVersion);

        let uptime = data.extract_map::<u64, _>(DataField::Uptime, Duration::from_secs);

        let hashrate = data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::TeraHash,
            algo: String::from("SHA256"),
        });
        let expected_hashrate =
            data.extract_map::<f64, _>(DataField::ExpectedHashrate, |f| HashRate {
                value: f,
                unit: HashRateUnit::TeraHash,
                algo: String::from("SHA256"),
            });

        let wattage = data.extract_map::<f64, _>(DataField::Wattage, Power::from_watts);

        let fluid_temperature =
            data.extract_map::<f64, _>(DataField::FluidTemperature, Temperature::from_celsius);

        let efficiency = match (hashrate.as_ref(), wattage.as_ref()) {
            (Some(hr), Some(w)) => {
                let hashrate_th = hr.value / 1000.0;
                Some(w.as_watts() / hashrate_th)
            }
            _ => None,
        };

        let hashboards: Vec<BoardData> = {
            let mut hashboards: Vec<BoardData> = Vec::new();
            let board_count = self.device_info.hardware.boards.unwrap_or(3);
            for idx in 0..board_count {
                let hashrate = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/hash-average", idx)))
                    .and_then(|val| val.as_f64())
                    .and_then(|f| {
                        Some(HashRate {
                            value: f,
                            unit: HashRateUnit::TeraHash,
                            algo: String::from("SHA256"),
                        })
                    });
                let expected_hashrate = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/factory-hash", idx)))
                    .and_then(|val| val.as_f64())
                    .and_then(|f| {
                        Some(HashRate {
                            value: f,
                            unit: HashRateUnit::TeraHash,
                            algo: String::from("SHA256"),
                        })
                    });
                let board_temperature = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/chip-temp-min", idx)))
                    .and_then(|val| val.as_f64())
                    .and_then(|f| Some(Temperature::from_celsius(f)));
                let intake_temperature = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/chip-temp-min", idx)))
                    .and_then(|val| val.as_f64())
                    .and_then(|f| Some(Temperature::from_celsius(f)));
                let outlet_temperature = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/chip-temp-max", idx)))
                    .and_then(|val| val.as_f64())
                    .and_then(|f| Some(Temperature::from_celsius(f)));
                let serial_number = data
                    .extract_nested::<String>(DataField::Hashboards, &format!("chipdata{}", idx));

                let working_chips = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/effective-chips", idx)))
                    .and_then(|val| val.as_u64())
                    .and_then(|u| Some(u as u16));
                let frequency = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/freq", idx)))
                    .and_then(|val| val.as_f64())
                    .and_then(|f| Some(Frequency::from_megahertz(f)));
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
                    voltage: None,
                    frequency,
                    tuned: Some(true),
                    active: Some(true),
                });
            }
            hashboards
        };

        // Get hardware specifications based on the miner model
        let miner_hardware = self.device_info.hardware.clone();

        MinerData {
            // Version information
            schema_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp,

            // Network identification
            ip: self.ip,
            mac,

            // Device identification
            device_info: self.device_info.clone(),
            serial_number: None,
            hostname,

            // Version information
            api_version,
            firmware_version,
            control_board_version,

            // Hashboard information
            expected_hashboards: miner_hardware.boards,
            hashboards,
            hashrate,
            expected_hashrate,

            // Chip information
            expected_chips: miner_hardware.chips,
            total_chips: miner_hardware.chips, // TODO

            // Cooling information
            expected_fans: miner_hardware.fans,
            fans: vec![], // TODO
            psu_fans: vec![],
            average_temperature: None, // TODO
            fluid_temperature,

            // Power information
            wattage,
            wattage_limit: None,
            efficiency,

            // Status information
            light_flashing: None,
            messages: vec![], // TODO
            uptime,
            is_mining: true, // TODO

            pools: vec![], // TODO
        }
    }

    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        let get_device_info_cmd: MinerCommand = MinerCommand::RPC {
            command: "get.device.info",
            parameters: None,
        };
        let get_miner_status_summary_cmd: MinerCommand = MinerCommand::RPC {
            command: "get.miner.status",
            parameters: Some(json!("summary")),
        };
        let get_miner_status_pools_cmd: MinerCommand = MinerCommand::RPC {
            command: "get.miner.status",
            parameters: Some(json!("pools")),
        };
        let get_miner_status_edevs_cmd: MinerCommand = MinerCommand::RPC {
            command: "get.miner.status",
            parameters: Some(json!("edevs")),
        };

        match data_field {
            DataField::Mac => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/network/mac"),
                },
            )],
            DataField::ApiVersion => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/api"),
                },
            )],
            DataField::FirmwareVersion => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/fwversion"),
                },
            )],
            DataField::ControlBoardVersion => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/platform"),
                },
            )],
            DataField::SerialNumber => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/miner/miner-sn"),
                },
            )],
            DataField::Hostname => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/network/hostname"),
                },
            )],
            DataField::LightFlashing => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/system/ledstatus"),
                },
            )],
            DataField::WattageLimit => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/miner/power-limit-set"),
                },
            )],
            DataField::PsuFans => vec![(
                get_device_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/power/fanspeed"),
                },
            )],
            DataField::Hashboards => vec![
                (
                    get_device_info_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/msg/miner"),
                    },
                ),
                (
                    get_miner_status_edevs_cmd,
                    DataExtractor {
                        func: get_by_key,
                        key: Some("msg"),
                    },
                ),
            ],
            DataField::Pools => vec![(
                get_miner_status_pools_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/pools"),
                },
            )],
            DataField::Uptime => vec![(
                get_miner_status_summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/summary/elapsed"),
                },
            )],
            DataField::Wattage => vec![(
                get_miner_status_summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/summary/power-realtime"),
                },
            )],
            DataField::Hashrate => vec![(
                get_miner_status_summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/summary/hash-realtime"),
                },
            )],
            DataField::ExpectedHashrate => vec![(
                get_miner_status_summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/summary/factory-hash"),
                },
            )],
            DataField::FluidTemperature => vec![(
                get_miner_status_summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/msg/summary/environment-temperature"),
                },
            )],
            _ => vec![],
        }
    }
}
