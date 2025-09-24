use crate::data::board::BoardData;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerMake, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::message::{MessageSeverity, MinerMessage};
use crate::data::pool::{PoolData, PoolURL};
use crate::miners::backends::traits::*;
use crate::miners::commands::MinerCommand;
use crate::miners::data::{
    DataCollector, DataExtensions, DataExtractor, DataField, DataLocation, get_by_pointer,
};
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};
use serde_json::Value;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use web::BraiinsWebAPI;

mod web;

#[derive(Debug)]
pub struct BraiinsV2507 {
    pub ip: IpAddr,
    pub web: BraiinsWebAPI,
    pub device_info: DeviceInfo,
}

impl BraiinsV2507 {
    pub fn new(ip: IpAddr, model: MinerModel) -> Self {
        BraiinsV2507 {
            ip,
            web: BraiinsWebAPI::new(ip),
            device_info: DeviceInfo::new(
                MinerMake::from(model),
                model,
                MinerFirmware::BraiinsOS,
                HashAlgorithm::SHA256,
            ),
        }
    }
}

#[async_trait]
impl APIClient for BraiinsV2507 {
    async fn get_api_result(&self, command: &MinerCommand) -> Result<Value> {
        match command {
            MinerCommand::WebAPI { .. } => self.web.get_api_result(command).await,
            _ => Err(anyhow!("Unsupported command type for Braiins API")),
        }
    }
}

impl GetDataLocations for BraiinsV2507 {
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        let network_cmd = MinerCommand::WebAPI {
            command: "network",
            parameters: None,
        };
        let version_cmd = MinerCommand::WebAPI {
            command: "version",
            parameters: None,
        };
        let miner_details_cmd = MinerCommand::WebAPI {
            command: "miner/details",
            parameters: None,
        };
        let locate_cmd = MinerCommand::WebAPI {
            command: "actions/locate",
            parameters: None,
        };
        let miner_stats_cmd = MinerCommand::WebAPI {
            command: "miner/stats",
            parameters: None,
        };
        let performance_tuner_state_cmd = MinerCommand::WebAPI {
            command: "performance/tuner-state",
            parameters: None,
        };
        let miner_errors_cmd = MinerCommand::WebAPI {
            command: "miner/errors",
            parameters: None,
        };
        let pools_cmd = MinerCommand::WebAPI {
            command: "pools",
            parameters: None,
        };
        let cooling_state_cmd = MinerCommand::WebAPI {
            command: "cooling/state",
            parameters: None,
        };
        let hashboards_cmd = MinerCommand::WebAPI {
            command: "miner/hw/hashboards",
            parameters: None,
        };

        match data_field {
            DataField::Mac => vec![(
                network_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/mac_address"),
                    tag: None,
                },
            )],
            DataField::Hostname => vec![(
                network_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/hostname"),
                    tag: None,
                },
            )],
            DataField::ApiVersion => vec![(
                version_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::FirmwareVersion => vec![(
                miner_details_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/bos_version/current"),
                    tag: None,
                },
            )],
            DataField::Hashrate => vec![(
                miner_stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner_stats/real_hashrate/last_5s/gigahash_per_second"),
                    tag: None,
                },
            )],
            DataField::ExpectedHashrate => vec![(
                miner_details_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/sticker_hashrate/gigahash_per_second"),
                    tag: None,
                },
            )],
            DataField::Fans => vec![(
                cooling_state_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/fans"),
                    tag: None,
                },
            )],
            DataField::Hashboards => vec![(
                hashboards_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/hashboards"),
                    tag: None,
                },
            )],
            DataField::LightFlashing => vec![(
                locate_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::IsMining => vec![(
                miner_details_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/status"),
                    tag: None,
                },
            )],
            DataField::Uptime => vec![(
                miner_details_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/system_uptime_s"),
                    tag: None,
                },
            )],
            DataField::ControlBoardVersion => vec![(
                miner_details_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/control_board_soc_family"),
                    tag: None,
                },
            )],
            DataField::Pools => vec![(
                pools_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/0/pools"), // assuming there is 1 pool group
                    tag: None,
                },
            )],
            DataField::Wattage => vec![(
                miner_stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/power_stats/approximated_consumption/watt"),
                    tag: None,
                },
            )],
            DataField::WattageLimit => vec![(
                performance_tuner_state_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/mode_state/powertargetmodestate/current_target/watt"),
                    tag: None,
                },
            )],
            DataField::SerialNumber => vec![(
                miner_details_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/serial_number"),
                    tag: None,
                },
            )],
            DataField::Messages => vec![(
                miner_errors_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/errors"),
                    tag: None,
                },
            )],
            _ => vec![],
        }
    }
}

impl GetIP for BraiinsV2507 {
    fn get_ip(&self) -> IpAddr {
        self.ip
    }
}

impl GetDeviceInfo for BraiinsV2507 {
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info
    }
}

impl CollectData for BraiinsV2507 {
    fn get_collector(&self) -> DataCollector<'_> {
        DataCollector::new(self)
    }
}

impl GetMAC for BraiinsV2507 {
    fn parse_mac(&self, data: &HashMap<DataField, Value>) -> Option<MacAddr> {
        data.extract::<String>(DataField::Mac)
            .and_then(|s| MacAddr::from_str(&s).ok())
    }
}

impl GetHostname for BraiinsV2507 {
    fn parse_hostname(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::Hostname)
    }
}

impl GetApiVersion for BraiinsV2507 {
    fn parse_api_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        let major = data.extract_nested::<f64>(DataField::ApiVersion, "major");
        let minor = data.extract_nested::<f64>(DataField::ApiVersion, "minor");
        let patch = data.extract_nested::<f64>(DataField::ApiVersion, "patch");

        Some(format!("{}.{}.{}", major?, minor?, patch?))
    }
}

impl GetFirmwareVersion for BraiinsV2507 {
    fn parse_firmware_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::FirmwareVersion)
    }
}

impl GetHashboards for BraiinsV2507 {
    fn parse_hashboards(&self, data: &HashMap<DataField, Value>) -> Vec<BoardData> {
        let mut hashboards: Vec<BoardData> = Vec::new();

        let chains_data = data.get(&DataField::Hashboards).and_then(|v| v.as_array());

        if let Some(chains_array) = chains_data {
            for (idx, chain) in chains_array.iter().enumerate() {
                let hashrate = chain
                    .pointer("/stats/real_hashrate/last_5s/gigahash_per_second")
                    .and_then(|v| v.as_f64())
                    .map(|f| HashRate {
                        value: f,
                        unit: HashRateUnit::GigaHash,
                        algo: String::from("SHA256"),
                    });
                let expected_hashrate = chain
                    .pointer("/stats/nominal_hashrate/gigahash_per_second")
                    .and_then(|v| v.as_f64())
                    .map(|f| HashRate {
                        value: f,
                        unit: HashRateUnit::GigaHash,
                        algo: String::from("SHA256"),
                    });

                let frequency = chain
                    .pointer("/current_frequency/hertz")
                    .and_then(|v| v.as_f64())
                    .map(Frequency::from_hertz);
                let voltage = chain
                    .pointer("/current_voltage/volt")
                    .and_then(|v| v.as_f64())
                    .map(Voltage::from_volts);
                let board_temperature = chain
                    .pointer("/board_temp/degree_c")
                    .and_then(|v| v.as_f64())
                    .map(Temperature::from_celsius);
                let chip_temperature = chain
                    .pointer("/highest_chip_temp/temperature/degree_c")
                    .and_then(|v| v.as_f64())
                    .map(Temperature::from_celsius);

                let working_chips = chain
                    .pointer("/chips_count")
                    .and_then(|v| v.as_u64())
                    .map(|u| u as u16);
                let active = chain.pointer("/enabled").and_then(|v| v.as_bool());
                let serial_number = chain
                    .pointer("/serial_number")
                    .and_then(|v| v.as_str())
                    .map(|u| u.to_string());

                hashboards.push(BoardData {
                    position: chain
                        .pointer("/id")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(idx as u64) as u8,
                    hashrate,
                    expected_hashrate,
                    board_temperature,
                    intake_temperature: chip_temperature,
                    outlet_temperature: chip_temperature,
                    expected_chips: self.device_info.hardware.chips,
                    working_chips,
                    serial_number,
                    chips: Vec::new(),
                    voltage,
                    frequency,
                    tuned: None, // Can maybe be parsed later from tuner status endpoint
                    active,
                });
            }
        }

        hashboards
    }
}

impl GetHashrate for BraiinsV2507 {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::GigaHash,
            algo: String::from("SHA256"),
        })
    }
}

impl GetExpectedHashrate for BraiinsV2507 {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::ExpectedHashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::GigaHash,
            algo: String::from("SHA256"),
        })
    }
}

impl GetFans for BraiinsV2507 {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        let mut fans: Vec<FanData> = Vec::new();

        if let Some(fans_data) = data.get(&DataField::Fans)
            && let Some(fans_array) = fans_data.as_array()
        {
            for (idx, fan) in fans_array.iter().enumerate() {
                if let Some(rpm) = fan.pointer("/rpm").and_then(|v| v.as_i64()) {
                    let pos = fan
                        .pointer("/position")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(idx as i64);
                    fans.push(FanData {
                        position: pos as i16,
                        rpm: Some(AngularVelocity::from_rpm(rpm as f64)),
                    });
                }
            }
        }

        fans
    }
}

impl GetLightFlashing for BraiinsV2507 {
    fn parse_light_flashing(&self, data: &HashMap<DataField, Value>) -> Option<bool> {
        data.extract::<bool>(DataField::LightFlashing)
    }
}

impl GetUptime for BraiinsV2507 {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.extract_map::<u64, _>(DataField::Uptime, Duration::from_secs)
    }
}

impl GetIsMining for BraiinsV2507 {
    fn parse_is_mining(&self, data: &HashMap<DataField, Value>) -> bool {
        // 1 -> Not Started
        // 2 -> Normal
        // 3 -> Paused
        // 4 -> Suspended
        // See: https://github.com/braiins/bos-plus-api/blob/ef28e752f80711c54d5587ec8f2cd838fdb34042/proto/bos/v1/miner.proto#L117-L124
        data.extract::<u64>(DataField::IsMining) == Some(2)
    }
}

impl GetPools for BraiinsV2507 {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolData> {
        let mut pools: Vec<PoolData> = Vec::new();

        if let Some(pools_data) = data.get(&DataField::Pools)
            && let Some(pools_array) = pools_data.as_array()
        {
            for (idx, pool) in pools_array.iter().enumerate() {
                let url = pool
                    .pointer("/url")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .map(PoolURL::from);

                let user = pool
                    .pointer("/user")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let accepted_shares = pool
                    .pointer("/stats/accepted_shares")
                    .and_then(|v| v.as_u64());
                let rejected_shares = pool
                    .pointer("/stats/rejected_shares")
                    .and_then(|v| v.as_u64());
                let active = pool.pointer("/active").and_then(|v| v.as_bool());
                let alive = pool.pointer("/alive").and_then(|v| v.as_bool());

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

impl GetSerialNumber for BraiinsV2507 {
    fn parse_serial_number(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::SerialNumber)
    }
}

impl GetControlBoardVersion for BraiinsV2507 {
    fn parse_control_board_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        let cb_type = data.extract::<u64>(DataField::ControlBoardVersion)?;
        match cb_type {
            0 => Some("Unknown".to_string()),
            1 => Some("CVITEK".to_string()),
            2 => Some("BBB".to_string()),
            3 => Some("AMLogic".to_string()),
            4 => Some("Xilinx".to_string()),
            5 => Some("BCB100".to_string()),
            _ => Some("Unknown".to_string()),
        }
    }
}

impl GetWattage for BraiinsV2507 {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<i64, _>(DataField::Wattage, |w| Power::from_watts(w as f64))
    }
}

impl GetWattageLimit for BraiinsV2507 {
    fn parse_wattage_limit(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<i64, _>(DataField::WattageLimit, |w| Power::from_watts(w as f64))
    }
}

impl GetFluidTemperature for BraiinsV2507 {}

impl GetPsuFans for BraiinsV2507 {}

impl GetMessages for BraiinsV2507 {
    fn parse_messages(&self, data: &HashMap<DataField, Value>) -> Vec<MinerMessage> {
        let mut messages: Vec<MinerMessage> = Vec::new();

        if let Some(errors_data) = data.get(&DataField::Messages)
            && let Some(errors_array) = errors_data.as_array()
        {
            for error in errors_array.iter() {
                let timestamp = error
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|dt| dt.parse::<DateTime<Utc>>().ok())
                    .map(|dt| dt.timestamp_millis() as u32);
                let message = error.get("message").and_then(|v| v.as_str());
                if let Some(ts) = timestamp {
                    messages.push(MinerMessage::new(
                        ts,
                        0, // They have codes, but they include a string
                        message.unwrap_or("Unknown error").to_string(),
                        MessageSeverity::Error,
                    ))
                }
            }
        };

        messages
    }
}

#[async_trait]
impl SetFaultLight for BraiinsV2507 {
    #[allow(unused_variables)]
    async fn set_fault_light(&self, fault: bool) -> Result<bool> {
        bail!("Unsupported command");
    }
}

#[async_trait]
impl SetPowerLimit for BraiinsV2507 {
    #[allow(unused_variables)]
    async fn set_power_limit(&self, limit: Power) -> Result<bool> {
        bail!("Unsupported command");
    }
}

#[async_trait]
impl Restart for BraiinsV2507 {
    async fn restart(&self) -> Result<bool> {
        bail!("Unsupported command");
    }
}

#[async_trait]
impl Pause for BraiinsV2507 {
    #[allow(unused_variables)]
    async fn pause(&self, at_time: Option<Duration>) -> Result<bool> {
        bail!("Unsupported command");
    }
}

#[async_trait]
impl Resume for BraiinsV2507 {
    #[allow(unused_variables)]
    async fn resume(&self, at_time: Option<Duration>) -> Result<bool> {
        bail!("Unsupported command");
    }
}
