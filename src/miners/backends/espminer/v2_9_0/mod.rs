use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};
use serde_json::Value;

use crate::data::board::{BoardData, ChipData};
use crate::data::device::MinerMake;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::message::{MessageSeverity, MinerMessage};
use crate::data::pool::{PoolData, PoolScheme, PoolURL};
use crate::miners::backends::traits::*;
use crate::miners::commands::MinerCommand;
use crate::miners::data::{
    DataCollector, DataExtensions, DataExtractor, DataField, DataLocation, get_by_key,
    get_by_pointer,
};
use web::ESPMinerWebAPI;
pub mod web;

#[derive(Debug)]
pub struct ESPMiner290 {
    ip: IpAddr,
    web: ESPMinerWebAPI,
    device_info: DeviceInfo,
}

impl ESPMiner290 {
    pub fn new(ip: IpAddr, model: MinerModel, firmware: MinerFirmware) -> Self {
        ESPMiner290 {
            ip,
            web: ESPMinerWebAPI::new(ip, 80),
            device_info: DeviceInfo::new(MinerMake::BitAxe, model, firmware, HashAlgorithm::SHA256),
        }
    }
}

#[async_trait]
impl GetDataLocations for ESPMiner290 {
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        let system_info_cmd: MinerCommand = MinerCommand::WebAPI {
            command: "system/info",
            parameters: None,
        };
        let asic_info_cmd: MinerCommand = MinerCommand::WebAPI {
            command: "system/asic",
            parameters: None,
        };

        match data_field {
            DataField::Mac => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("macAddr"),
                },
            )],
            DataField::Hostname => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("hostname"),
                },
            )],
            DataField::FirmwareVersion => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("version"),
                },
            )],
            DataField::ApiVersion => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("version"),
                },
            )],
            DataField::ControlBoardVersion => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("boardVersion"),
                },
            )],
            DataField::ExpectedHashrate => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("expectedHashrate"),
                },
            )],
            DataField::Hashboards => vec![
                (
                    system_info_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some(""),
                    },
                ),
                (
                    asic_info_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some(""),
                    },
                ),
            ],
            DataField::Hashrate => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("hashRate"),
                },
            )],
            DataField::Fans => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("fanrpm"),
                },
            )],
            DataField::AverageTemperature => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("temp"),
                },
            )],
            DataField::Wattage => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("power"),
                },
            )],
            DataField::Uptime => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_key,
                    key: Some("uptimeSeconds"),
                },
            )],
            DataField::Pools => vec![(
                system_info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                },
            )],
            _ => vec![],
        }
    }
}

impl GetIP for ESPMiner290 {
    fn get_ip(&self) -> IpAddr {
        self.ip
    }
}
impl GetDeviceInfo for ESPMiner290 {
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }
}

impl CollectData for ESPMiner290 {
    fn get_collector(&self) -> DataCollector {
        DataCollector::new(self, &self.web)
    }
}

impl GetMAC for ESPMiner290 {
    fn parse_mac(&self, data: &HashMap<DataField, Value>) -> Option<MacAddr> {
        data.extract::<String>(DataField::Mac)
            .and_then(|s| MacAddr::from_str(&s).ok())
    }
}

impl GetSerialNumber for ESPMiner290 {
    // N/A
}
impl GetHostname for ESPMiner290 {
    fn parse_hostname(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::Hostname)
    }
}
impl GetApiVersion for ESPMiner290 {
    fn parse_api_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::ApiVersion)
    }
}
impl GetFirmwareVersion for ESPMiner290 {
    fn parse_firmware_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::FirmwareVersion)
    }
}
impl GetControlBoardVersion for ESPMiner290 {
    fn parse_control_board_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::ControlBoardVersion)
    }
}
impl GetHashboards for ESPMiner290 {
    fn parse_hashboards(&self, data: &HashMap<DataField, Value>) -> Vec<BoardData> {
        // Extract nested values with type conversion
        let board_voltage = data.extract_nested_map::<f64, _>(
            DataField::Hashboards,
            "voltage",
            Voltage::from_millivolts,
        );

        let board_temperature = data.extract_nested_map::<f64, _>(
            DataField::Hashboards,
            "vrTemp",
            Temperature::from_celsius,
        );

        let board_frequency = data.extract_nested_map::<f64, _>(
            DataField::Hashboards,
            "frequency",
            Frequency::from_megahertz,
        );

        let chip_temperature = data.extract_nested_map::<f64, _>(
            DataField::Hashboards,
            "temp",
            Temperature::from_celsius,
        );

        let expected_hashrate = Some(HashRate {
            value: data.extract_nested_or::<f64>(DataField::Hashboards, "expectedHashrate", 0.0),
            unit: HashRateUnit::GigaHash,
            algo: "SHA256".to_string(),
        });

        let board_hashrate = Some(HashRate {
            value: data.extract_nested_or::<f64>(DataField::Hashboards, "hashRate", 0.0),
            unit: HashRateUnit::GigaHash,
            algo: "SHA256".to_string(),
        });

        let total_chips =
            data.extract_nested_map::<u64, _>(DataField::Hashboards, "asicCount", |u| u as u16);

        let chip_info = ChipData {
            position: 0,
            temperature: chip_temperature,
            voltage: board_voltage,
            frequency: board_frequency,
            tuned: Some(true),
            working: Some(true),
            hashrate: board_hashrate.clone(),
        };

        let board_data = BoardData {
            position: 0,
            hashrate: board_hashrate,
            expected_hashrate,
            board_temperature,
            intake_temperature: board_temperature,
            outlet_temperature: board_temperature,
            expected_chips: self.device_info.hardware.chips,
            working_chips: total_chips,
            serial_number: None,
            chips: vec![chip_info],
            voltage: board_voltage,
            frequency: board_frequency,
            tuned: Some(true),
            active: Some(true),
        };

        vec![board_data]
    }
}
impl GetHashrate for ESPMiner290 {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::TeraHash,
            algo: String::from("SHA256"),
        })
    }
}
impl GetExpectedHashrate for ESPMiner290 {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::TeraHash,
            algo: String::from("SHA256"),
        })
    }
}
impl GetFans for ESPMiner290 {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        data.extract_map_or::<f64, _>(DataField::Fans, Vec::new(), |f| {
            vec![FanData {
                position: 0,
                rpm: Some(AngularVelocity::from_rpm(f)),
            }]
        })
    }
}
impl GetPsuFans for ESPMiner290 {
    // N/A
}
impl GetFluidTemperature for ESPMiner290 {
    // N/A
}
impl GetWattage for ESPMiner290 {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<f64, _>(DataField::Wattage, Power::from_watts)
    }
}
impl GetWattageLimit for ESPMiner290 {
    // N/A
}
impl GetLightFlashing for ESPMiner290 {
    // N/A
}
impl GetMessages for ESPMiner290 {
    fn parse_messages(&self, data: &HashMap<DataField, Value>) -> Vec<MinerMessage> {
        let mut messages = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_secs();

        let is_overheating = data.extract_nested::<bool>(DataField::Hashboards, "overheat_mode");

        if let Some(true) = is_overheating {
            messages.push(MinerMessage {
                timestamp: timestamp as u32,
                code: 0u64,
                message: "Overheat Mode is Enabled!".to_string(),
                severity: MessageSeverity::Warning,
            });
        };
        messages
    }
}
impl GetUptime for ESPMiner290 {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.extract_map::<u64, _>(DataField::Uptime, Duration::from_secs)
    }
}
impl GetIsMining for ESPMiner290 {
    fn parse_is_mining(&self, data: &HashMap<DataField, Value>) -> bool {
        let hashrate = self.parse_hashrate(data);
        hashrate.as_ref().is_some_and(|hr| hr.value > 0.0)
    }
}
impl GetPools for ESPMiner290 {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolData> {
        let main_url =
            data.extract_nested_or::<String>(DataField::Pools, "stratumURL", String::new());
        let main_port = data.extract_nested_or::<u64>(DataField::Pools, "stratumPort", 0);
        let accepted_share = data.extract_nested::<u64>(DataField::Pools, "sharesAccepted");
        let rejected_share = data.extract_nested::<u64>(DataField::Pools, "sharesRejected");
        let main_user = data.extract_nested::<String>(DataField::Pools, "stratumUser");

        let is_using_fallback =
            data.extract_nested_or::<bool>(DataField::Pools, "isUsingFallbackStratum", false);

        let main_pool_url = PoolURL {
            scheme: PoolScheme::StratumV1,
            host: main_url,
            port: main_port as u16,
            pubkey: None,
        };

        let main_pool_data = PoolData {
            position: Some(0),
            url: Some(main_pool_url),
            accepted_shares: accepted_share,
            rejected_shares: rejected_share,
            active: Some(!is_using_fallback),
            alive: None,
            user: main_user,
        };

        // Extract fallback pool data
        let fallback_url =
            data.extract_nested_or::<String>(DataField::Pools, "fallbackStratumURL", String::new());
        let fallback_port =
            data.extract_nested_or::<u64>(DataField::Pools, "fallbackStratumPort", 0);
        let fallback_user = data.extract_nested(DataField::Pools, "fallbackStratumUser");
        let fallback_pool_url = PoolURL {
            scheme: PoolScheme::StratumV1,
            host: fallback_url,
            port: fallback_port as u16,
            pubkey: None,
        };

        let fallback_pool_data = PoolData {
            position: Some(1),
            url: Some(fallback_pool_url),
            accepted_shares: accepted_share,
            rejected_shares: rejected_share,
            active: Some(is_using_fallback),
            alive: None,
            user: fallback_user,
        };

        vec![main_pool_data, fallback_pool_data]
    }
}
