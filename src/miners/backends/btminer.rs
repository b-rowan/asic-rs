use super::traits::GetMinerData;
use crate::data::board::BoardData;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::miner::MinerData;
use crate::data::pool::{PoolData, PoolURL};
use crate::miners::api::rpc::errors::RPCError;
use crate::miners::api::rpc::{btminer::BTMinerV3RPC, traits::SendRPCCommand};
use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct BTMinerV3Backend {
    pub ip: IpAddr,
    pub rpc: BTMinerV3RPC,
    pub device_info: DeviceInfo,
}

impl BTMinerV3Backend {
    pub fn new(ip: IpAddr, model: MinerModel) -> Self {
        BTMinerV3Backend {
            ip,
            rpc: BTMinerV3RPC::new(ip, None),
            device_info: DeviceInfo::new(model, MinerFirmware::Stock, HashAlgorithm::SHA256),
        }
    }
    pub async fn get_device_info(&self) -> Result<GetDeviceInfo, RPCError> {
        self.rpc
            .send_command::<GetDeviceInfo, ()>("get.device.info", None)
            .await
    }
    pub async fn get_miner_status_summary(&self) -> Result<GetMinerStatusSummary, RPCError> {
        self.rpc
            .send_command::<GetMinerStatusSummary, &str>("get.miner.status", Some("summary"))
            .await
    }
    pub async fn get_miner_status_pools(&self) -> Result<GetMinerStatusPools, RPCError> {
        self.rpc
            .send_command::<GetMinerStatusPools, &str>("get.miner.status", Some("pools"))
            .await
    }
    pub async fn get_miner_status_edevs(&self) -> Result<GetMinerStatusEDevs, RPCError> {
        self.rpc
            .send_command::<GetMinerStatusEDevs, &str>("get.miner.status", Some("edevs"))
            .await
    }
}

#[async_trait]
impl GetMinerData for BTMinerV3Backend {
    async fn get_data(&self) -> MinerData {
        let (device_info, miner_status_summary, miner_status_pools, miner_status_edevs) = tokio::join!(
            self.get_device_info(),
            self.get_miner_status_summary(),
            self.get_miner_status_pools(),
            self.get_miner_status_edevs(),
        );

        // construct hashboard info
        let mut boards: Vec<BoardData> = Vec::new();
        for position in 0..self.device_info.hardware.boards.unwrap_or(0) {
            let hashrate = match &miner_status_edevs {
                Ok(edevs) => edevs.board_hashrates.get(position as usize).cloned(),
                _ => None,
            };
            let expected_hashrate = match &miner_status_edevs {
                Ok(edevs) => edevs
                    .board_expected_hashrates
                    .get(position as usize)
                    .cloned(),
                _ => None,
            };
            let outlet_temperature = match &miner_status_edevs {
                Ok(edevs) => edevs
                    .board_outlet_temperatures
                    .get(position as usize)
                    .cloned(),
                _ => None,
            };
            let intake_temperature = match &miner_status_edevs {
                Ok(edevs) => edevs
                    .board_intake_temperatures
                    .get(position as usize)
                    .cloned(),
                _ => None,
            };
            let working_chips = match &miner_status_edevs {
                Ok(edevs) => edevs.board_working_chips.get(position as usize).cloned(),
                _ => None,
            };
            let frequency = match &miner_status_edevs {
                Ok(edevs) => edevs.board_freqs.get(position as usize).cloned(),
                _ => None,
            };
            let serial_number = match &device_info {
                Ok(info) => info.board_sns.get(position as usize).cloned(),
                _ => None,
            };

            boards.push(BoardData {
                position,
                hashrate,
                expected_hashrate,
                board_temperature: None,
                intake_temperature,
                outlet_temperature,
                expected_chips: self.device_info.hardware.chips,
                working_chips,
                serial_number,
                chips: Vec::new(),
                voltage: match &device_info {
                    Ok(info) => info.voltage,
                    _ => None,
                },
                frequency,
                tuned: None,
                active: Some(true),
            })
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_secs();

        let hashrate = match &miner_status_summary {
            Ok(summary) => summary.hashrate.clone(),
            _ => None,
        };
        let wattage = match &miner_status_summary {
            Ok(summary) => summary.wattage.clone(),
            _ => None,
        };
        let efficiency = match (hashrate.clone(), wattage.clone()) {
            (Some(hr), Some(w)) => Some(w / hr),
            _ => None,
        };

        MinerData {
            schema_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp,
            ip: self.ip.clone(),
            mac: match &device_info {
                Ok(info) => info.mac.clone(),
                _ => None,
            },
            device_info: self.device_info.clone(),
            serial_number: match &device_info {
                Ok(info) => info.serial_number.clone(),
                _ => None,
            },
            hostname: match &device_info {
                Ok(info) => info.hostname.clone(),
                _ => None,
            },
            api_version: match &device_info {
                Ok(info) => info.api_version.clone(),
                _ => None,
            },
            firmware_version: match &device_info {
                Ok(info) => info.fw_version.clone(),
                _ => None,
            },
            control_board_version: match &device_info {
                Ok(info) => info.control_board_version.clone(),
                _ => None,
            },
            expected_hashboards: self.device_info.hardware.boards.clone(),
            hashboards: boards.clone(),
            hashrate,
            expected_chips: match (
                self.device_info.hardware.chips,
                self.device_info.hardware.boards,
            ) {
                (Some(chips), Some(boards)) => Some(chips * boards as u16),
                (Some(chips), _) => Some(chips.clone()),
                _ => None,
            },
            total_chips: Some(boards.iter().filter_map(|b| b.working_chips).sum()),
            expected_fans: self.device_info.hardware.fans.clone(),
            fans: match &miner_status_summary {
                Ok(summary) => summary.fans.clone(),
                _ => Vec::new(),
            },
            psu_fans: match &device_info {
                Ok(info) => info.psu_fans.clone(),
                _ => Vec::new(),
            },
            average_temperature: {
                let (sum, count) = boards
                    .iter()
                    .filter_map(|b| b.board_temperature.as_ref()) // or .intake_temperature, etc.
                    .fold((0.0, 0), |(sum, count), temp| {
                        (sum + temp.as_celsius(), count + 1)
                    });

                if count > 0 {
                    Some(Temperature::from_celsius(sum / count as f64))
                } else {
                    None
                }
            },
            fluid_temperature: match &miner_status_summary {
                Ok(summary) => summary.fluid_temperature.clone(),
                _ => None,
            },
            wattage,
            wattage_limit: match &device_info {
                Ok(info) => info.wattage_limit.clone(),
                _ => None,
            },
            efficiency,
            light_flashing: match &device_info {
                Ok(info) => info.light_flashing.clone(),
                _ => None,
            },
            messages: Vec::new(),
            uptime: match &miner_status_summary {
                Ok(summary) => summary.uptime.clone(),
                _ => None,
            },
            is_mining: true,
            pools: match &miner_status_pools {
                Ok(pools) => pools.pools.clone(),
                _ => Vec::new(),
            },
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
                rpm: AngularVelocity::from_rpm(f),
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
                    rpm: AngularVelocity::from_rpm(fan.unwrap()),
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
