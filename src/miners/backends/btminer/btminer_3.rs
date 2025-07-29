use crate::data::board::BoardData;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerMake, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
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
    async fn get_data(&self) -> MinerData {
        let mut collector = DataCollector::new(self, &self.web);
        let data = collector.collect_all().await;

        // Extract basic string fields
        let mac = data
            .extract::<String>(DataField::Mac)
            .and_then(|s| MacAddr::from_str(&s).ok());

        let hostname = data.extract::<String>(DataField::Hostname);
        let api_version = data.extract::<String>(DataField::ApiVersion);
        let firmware_version = data.extract::<String>(DataField::FirmwareVersion);
        let control_board_version = data.extract::<String>(DataField::ControlBoardVersion);

        // Extract hashrate and convert to HashRate structure
        let hashrate = data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::GigaHash,
            algo: String::from("SHA256"),
        });

        // Extract numeric values with conversions
        let total_chips = data.extract_map::<u64, _>(DataField::TotalChips, |u| u as u16);
        let wattage = data.extract_map::<f64, _>(DataField::Wattage, Power::from_watts);
        let average_temperature =
            data.extract_map::<f64, _>(DataField::AverageTemperature, Temperature::from_celsius);

        let efficiency = match (hashrate.as_ref(), wattage.as_ref()) {
            (Some(hr), Some(w)) => {
                let hashrate_th = hr.value / 1000.0;
                Some(w.as_watts() / hashrate_th)
            }
            _ => None,
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_secs();

        // Extract fan data with the default value if missing
        let fans = data.extract_map_or::<f64, _>(DataField::Fans, Vec::new(), |f| {
            vec![FanData {
                position: 0,
                rpm: Some(AngularVelocity::from_rpm(f)),
            }]
        });

        // Extract uptime
        let uptime = data.extract_map::<u64, _>(DataField::Uptime, Duration::from_secs);

        // Determine if the miner is actively mining based on hashrate
        let is_mining = hashrate.as_ref().map_or(false, |hr| hr.value > 0.0);

        // Get hardware specifications based on the miner model
        let miner_hardware = self.device_info.hardware.clone();

        let hashboards = {
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
                value: data.extract_nested_or::<f64>(
                    DataField::Hashboards,
                    "expectedHashrate",
                    0.0,
                ),
                unit: HashRateUnit::GigaHash,
                algo: "SHA256".to_string(),
            });

            let board_hashrate = hashrate.clone();

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
                expected_chips: miner_hardware.chips,
                working_chips: total_chips,
                serial_number: None,
                chips: vec![chip_info],
                voltage: board_voltage,
                frequency: board_frequency,
                tuned: Some(true),
                active: Some(true),
            };

            vec![board_data]
        };

        let pools = {
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
            let fallback_url = data.extract_nested_or::<String>(
                DataField::Pools,
                "fallbackStratumURL",
                String::new(),
            );
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
        };

        let mut messages = Vec::new();

        let is_overheating = data.extract_nested::<bool>(DataField::Hashboards, "overheat_mode");

        if let Some(true) = is_overheating {
            messages.push(MinerMessage {
                timestamp: timestamp as u32,
                code: 0u64,
                message: "Overheat Mode is Enabled!".to_string(),
                severity: MessageSeverity::Warning,
            });
        }

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

            // Chip information
            expected_chips: miner_hardware.chips,
            total_chips,

            // Cooling information
            expected_fans: miner_hardware.fans,
            fans,
            psu_fans: vec![],
            average_temperature,
            fluid_temperature: None,

            // Power information
            wattage,
            wattage_limit: None,
            efficiency,

            // Status information
            light_flashing: None,
            messages,
            uptime,
            is_mining,

            pools,
        }
    }

    fn get_locations(&self, data_field: DataField) -> &'static [DataLocation] {
        const SYSTEM_INFO_CMD: &str = "system/info";

        match data_field {
            DataField::Mac => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("macAddr"),
                },
            )],
            DataField::Hostname => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("hostname"),
                },
            )],
            DataField::FirmwareVersion => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("version"),
                },
            )],
            DataField::ApiVersion => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("version"),
                },
            )],
            DataField::ControlBoardVersion => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("boardVersion"),
                },
            )],
            DataField::Hashboards => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                },
            )],
            DataField::Hashrate => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("hashRate"),
                },
            )],
            DataField::TotalChips => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("asicCount"),
                },
            )],
            DataField::Fans => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("fanrpm"),
                },
            )],
            DataField::AverageTemperature => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("temp"),
                },
            )],
            DataField::Wattage => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("power"),
                },
            )],
            DataField::Uptime => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_key,
                    key: Some("uptimeSeconds"),
                },
            )],
            DataField::Pools => &[(
                SYSTEM_INFO_CMD,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                },
            )],
            _ => &[],
        }
    }
}

#[derive(Debug)]
pub struct GetDeviceInfo {
    pub api_version: Option<String>,
    pub fw_version: Option<String>,
    pub control_board_version: Option<String>,
    pub mac: Option<MacAddr>,
    pub serial_number: Option<String>,
    pub hostname: Option<String>,
    pub psu_fans: Vec<FanData>,
    pub light_flashing: Option<bool>,
    pub wattage_limit: Option<Power>,
    pub voltage: Option<Voltage>,
    pub board_sns: Vec<String>,
}

impl<'de> Deserialize<'de> for GetDeviceInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer)?;

        let api_version = value["msg"]["system"]["api"]
            .as_str()
            .map(|s| s.to_string());

        let fw_version = value["msg"]["system"]["fwversion"]
            .as_str()
            .map(|s| s.to_string());

        let control_board_version = value["msg"]["system"]["platform"]
            .as_str()
            .map(|s| s.to_string());

        let mac = value["msg"]["network"]["mac"]
            .as_str()
            .and_then(|s| MacAddr::from_str(s).ok());

        let serial_number = value["msg"]["miner"]["miner-sn"]
            .as_str()
            .map(|s| s.to_string());

        let hostname = value["msg"]["network"]["hostname"]
            .as_str()
            .map(|s| s.to_string());

        let light_flashing = value["msg"]["system"]["ledstatus"]
            .as_str()
            .map(|s| s != "auto");

        let wattage_limit = value["msg"]["miner"]["power-limit-set"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| Power::from_watts(f));

        let voltage = value["msg"]["power"]["vout"]
            .as_f64()
            .map(|f| Voltage::from_millivolts(f));

        let mut psu_fans: Vec<FanData> = Vec::new();

        value["msg"]["power"]["fanspeed"].as_f64().map(|f| {
            psu_fans.push(FanData {
                position: 0,
                rpm: Some(AngularVelocity::from_rpm(f)),
            })
        });

        let mut board_sns: Vec<String> = Vec::new();

        for idx in 0..3 {
            let board_sn = value["msg"]["miner"][format!("pcbsn{}", idx)].as_str();
            if board_sn.is_some() {
                board_sns.push(board_sn.unwrap().to_owned());
            }
        }

        Ok(Self {
            api_version,
            fw_version,
            control_board_version,
            mac,
            serial_number,
            hostname,
            psu_fans,
            light_flashing,
            wattage_limit,
            voltage,
            board_sns,
        })
    }
}

#[derive(Debug)]
pub struct GetMinerStatusSummary {
    pub uptime: Option<Duration>,
    pub wattage: Option<Power>,
    pub hashrate: Option<HashRate>,
    pub expected_hashrate: Option<HashRate>,
    pub fluid_temperature: Option<Temperature>,
    pub fans: Vec<FanData>,
}

impl<'de> Deserialize<'de> for GetMinerStatusSummary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer)?;

        let uptime = value["msg"]["summary"]["elapsed"]
            .as_u64()
            .map(|i| Duration::from_secs(i));

        let wattage = value["msg"]["summary"]["power-realtime"]
            .as_f64()
            .map(|f| Power::from_watts(f));

        let hashrate = value["msg"]["summary"]["hash-realtime"]
            .as_f64()
            .map(|f| HashRate {
                value: f,
                unit: HashRateUnit::TeraHash,
                algo: String::from("SHA256"),
            });

        let expected_hashrate =
            value["msg"]["summary"]["factory-hash"]
                .as_f64()
                .map(|f| HashRate {
                    value: f,
                    unit: HashRateUnit::TeraHash,
                    algo: String::from("SHA256"),
                });

        let fluid_temperature = value["msg"]["summary"]["environment-temperature"]
            .as_f64()
            .map(|f| Temperature::from_celsius(f));

        let mut fans: Vec<FanData> = Vec::new();

        for (idx, direction) in ["in", "out"].iter().enumerate() {
            let fan = value["msg"]["summary"][format!("fan-speed-{}", direction)].as_f64();
            if fan.is_some() {
                fans.push(FanData {
                    position: idx as i16,
                    rpm: Some(AngularVelocity::from_rpm(fan.unwrap())),
                });
            }
        }

        Ok(Self {
            uptime,
            wattage,
            hashrate,
            expected_hashrate,
            fluid_temperature,
            fans,
        })
    }
}

#[derive(Debug)]
pub struct GetMinerStatusPools {
    pools: Vec<PoolData>,
}

impl<'de> Deserialize<'de> for GetMinerStatusPools {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer)?;

        let pool_data = value["msg"]["pools"].as_array();

        let mut pools: Vec<PoolData> = Vec::new();
        pool_data.map(|p| {
            for pool in p.iter() {
                let position = pool["id"].as_u64().map(|u| (u - 1) as u16);
                let url = pool["url"].as_str().map(|s| PoolURL::from(s.to_string()));
                let alive = pool["status"].as_str().map(|s| s == "alive");
                let active = pool["stratum-active"].as_bool();
                let user = pool["account"].as_str().map(|s| s.to_string());

                pools.push(PoolData {
                    position,
                    url,
                    alive,
                    active,
                    user,
                    accepted_shares: None,
                    rejected_shares: None,
                });
            }
        });

        Ok(Self { pools })
    }
}

#[derive(Debug)]
pub struct GetMinerStatusEDevs {
    board_intake_temperatures: Vec<Temperature>,
    board_outlet_temperatures: Vec<Temperature>,
    board_working_chips: Vec<u16>,
    board_hashrates: Vec<HashRate>,
    board_expected_hashrates: Vec<HashRate>,
    board_freqs: Vec<Frequency>,
}

impl<'de> Deserialize<'de> for GetMinerStatusEDevs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer)?;

        let mut board_intake_temperatures: Vec<Temperature> = Vec::new();
        let mut board_outlet_temperatures: Vec<Temperature> = Vec::new();
        let mut board_working_chips: Vec<u16> = Vec::new();
        let mut board_hashrates: Vec<HashRate> = Vec::new();
        let mut board_expected_hashrates: Vec<HashRate> = Vec::new();
        let mut board_freqs: Vec<Frequency> = Vec::new();

        value["msg"]["edevs"].as_array().map(|devices| {
            for device in devices.iter() {
                device["chip-temp-min"]
                    .as_f64()
                    .map(|f| board_intake_temperatures.push(Temperature::from_celsius(f)));
                device["chip-temp-max"]
                    .as_f64()
                    .map(|f| board_outlet_temperatures.push(Temperature::from_celsius(f)));
                device["effective-chips"]
                    .as_u64()
                    .map(|u| board_working_chips.push(u as u16));
                device["hash-average"].as_f64().map(|f| {
                    board_hashrates.push(HashRate {
                        value: f,
                        unit: HashRateUnit::TeraHash,
                        algo: String::from("SHA256"),
                    })
                });
                device["factory-hash"].as_f64().map(|f| {
                    board_expected_hashrates.push(HashRate {
                        value: f,
                        unit: HashRateUnit::TeraHash,
                        algo: String::from("SHA256"),
                    })
                });
                device["freq"]
                    .as_f64()
                    .map(|f| board_freqs.push(Frequency::from_megahertz(f)));
            }
        });

        Ok(Self {
            board_intake_temperatures,
            board_outlet_temperatures,
            board_working_chips,
            board_hashrates,
            board_expected_hashrates,
            board_freqs,
        })
    }
}
