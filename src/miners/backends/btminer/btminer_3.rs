use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature};
use serde_json::json;

use crate::data::board::BoardData;
use crate::data::device::MinerMake;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::miner::MinerData;
use crate::data::pool::{PoolData, PoolURL};
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

        let mut fans: Vec<FanData> = Vec::new();

        for (idx, direction) in ["in", "out"].iter().enumerate() {
            let fan = data.extract_nested_map::<f64, _>(
                DataField::Fans,
                &format!("fan-speed-{}", direction),
                |rpm| FanData {
                    position: idx as i16,
                    rpm: Some(AngularVelocity::from_rpm(rpm)),
                },
            );
            if fan.is_some() {
                fans.push(fan.unwrap());
            }
        }

        let mut psu_fans: Vec<FanData> = Vec::new();

        let psu_fan = data.extract_map::<f64, _>(DataField::Fans, |rpm| FanData {
            position: 0i16,
            rpm: Some(AngularVelocity::from_rpm(rpm)),
        });
        if psu_fan.is_some() {
            psu_fans.push(psu_fan.unwrap());
        }

        let hashboards: Vec<BoardData> = {
            let mut hashboards: Vec<BoardData> = Vec::new();
            let board_count = self.device_info.hardware.boards.unwrap_or(3);
            for idx in 0..board_count {
                let hashrate = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/hash-average", idx)))
                    .and_then(|val| val.as_f64()).map(|f| HashRate {
                            value: f,
                            unit: HashRateUnit::TeraHash,
                            algo: String::from("SHA256"),
                        });
                let expected_hashrate = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/factory-hash", idx)))
                    .and_then(|val| val.as_f64()).map(|f| HashRate {
                            value: f,
                            unit: HashRateUnit::TeraHash,
                            algo: String::from("SHA256"),
                        });
                let board_temperature = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/chip-temp-min", idx)))
                    .and_then(|val| val.as_f64()).map(Temperature::from_celsius);
                let intake_temperature = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/chip-temp-min", idx)))
                    .and_then(|val| val.as_f64()).map(Temperature::from_celsius);
                let outlet_temperature = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/chip-temp-max", idx)))
                    .and_then(|val| val.as_f64()).map(Temperature::from_celsius);
                let serial_number =
                    data.extract_nested::<String>(DataField::Hashboards, &format!("pcdsn{}", idx));

                let working_chips = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/effective-chips", idx)))
                    .and_then(|val| val.as_u64()).map(|u| u as u16);
                let frequency = data
                    .get(&DataField::Hashboards)
                    .and_then(|val| val.pointer(&format!("/edevs/{}/freq", idx)))
                    .and_then(|val| val.as_f64()).map(Frequency::from_megahertz);

                let active =
                    Some(hashrate.clone().map(|h| h.value).unwrap_or(0f64) > 0f64);
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
        };
        let pools: Vec<PoolData> = {
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
                        .and_then(|val| val.pointer(&format!("/{}/account", idx))).map(|val| String::from(val.as_str().unwrap_or("")));

                    let alive = data
                        .get(&DataField::Pools)
                        .and_then(|val| val.pointer(&format!("/{}/status", idx))).map(|val| val.as_str()).map(|val| val == Some("alive"));

                    let active = data
                        .get(&DataField::Pools)
                        .and_then(|val| val.pointer(&format!("/{}/stratum-active", idx)))
                        .and_then(|val| val.as_bool());

                    let url = data
                        .get(&DataField::Pools)
                        .and_then(|val| val.pointer(&format!("/{}/url", idx))).map(|val| PoolURL::from(String::from(val.as_str().unwrap_or(""))));

                    pools.push(PoolData {
                        position: Some(idx as u16),
                        url,
                        accepted_shares: None,
                        rejected_shares: None,
                        active,
                        alive,
                        user,
                    });
                }
            }
            pools
        };

        let total_chips = hashboards.clone().iter().map(|b| b.working_chips).sum();

        let average_temperature = {
            let board_temps = hashboards
                .iter()
                .map(|b| b.board_temperature)
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().as_celsius())
                .collect::<Vec<f64>>();
            if !board_temps.is_empty() {
                Some(Temperature::from_celsius(
                    board_temps.iter().sum::<f64>() / hashboards.len() as f64,
                ))
            } else {
                None
            }
        };

        let wattage_limit =
            data.extract_map::<f64, _>(DataField::FluidTemperature, Power::from_watts);

        let light_flashing =
            data.extract_map::<String, _>(DataField::LightFlashing, |l| l != "auto");

        // Get hardware specifications based on the miner model
        let miner_hardware = self.device_info.hardware;

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
            total_chips,

            // Cooling information
            expected_fans: miner_hardware.fans,
            fans,
            psu_fans,
            average_temperature,
            fluid_temperature,

            // Power information
            wattage,
            wattage_limit,
            efficiency,

            // Status information
            light_flashing,
            messages: vec![], // TODO
            uptime,
            is_mining: true, // TODO

            pools,
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
