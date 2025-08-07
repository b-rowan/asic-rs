#![allow(dead_code)]
pub mod rpc;

use crate::data::board::{BoardData, ChipData};
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

use crate::miners::api::RPCAPIClient;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Power, Temperature, Voltage};
use rpc::AvalonMinerRPCAPI;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct AvalonMiner {
    ip: IpAddr,
    rpc: AvalonMinerRPCAPI,
    device_info: DeviceInfo,
}

impl AvalonMiner {
    pub fn new(ip: IpAddr, model: MinerModel, miner_firmware: MinerFirmware) -> Self {
        Self {
            ip,
            rpc: AvalonMinerRPCAPI::new(ip),
            device_info: DeviceInfo::new(
                MinerMake::AvalonMiner,
                model,
                miner_firmware,
                HashAlgorithm::SHA256,
            ),
        }
    }

    /// Reboot the miner
    pub async fn reboot(&self) -> Result<bool> {
        let data = self.rpc.send_command("restart", false, None).await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_str()) {
            return Ok(status == "RESTART");
        }

        Ok(false)
    }
}
#[async_trait]
impl Pause for AvalonMiner {
    async fn pause(&self, after: Option<Duration>) -> Result<bool> {
        let offset = after.unwrap_or(Duration::from_secs(5));
        let shutdown_time = SystemTime::now() + offset;

        let timestamp = shutdown_time
            .duration_since(UNIX_EPOCH)
            .expect("Shutdown time is before UNIX epoch")
            .as_secs();

        let data = self
            .rpc
            .send_command(
                "ascset",
                false,
                Some(json!(["0", format!("softoff,1:{}", timestamp)])),
            )
            .await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_array()) {
            if !status.is_empty() {
                if let Some(status_code) = status[0].get("STATUS").and_then(|s| s.as_str()) {
                    if status_code == "I" {
                        if let Some(msg) = status[0].get("Msg").and_then(|m| m.as_str()) {
                            return Ok(msg.contains("success softoff"));
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}
#[async_trait]
impl Resume for AvalonMiner {
    async fn resume(&self, after: Option<Duration>) -> Result<bool> {
        let offset = after.unwrap_or(Duration::from_secs(5));
        let shutdown_time = SystemTime::now() + offset;

        let timestamp = shutdown_time
            .duration_since(UNIX_EPOCH)
            .expect("Shutdown time is before UNIX epoch")
            .as_secs();

        let data = self
            .rpc
            .send_command(
                "ascset",
                false,
                Some(json!(["0", format!("softon,1:{}", timestamp)])),
            )
            .await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_array()) {
            if !status.is_empty() {
                if let Some(status_code) = status[0].get("STATUS").and_then(|s| s.as_str()) {
                    if status_code == "I" {
                        if let Some(msg) = status[0].get("Msg").and_then(|m| m.as_str()) {
                            return Ok(msg.contains("success softon"));
                        }
                    }
                }
            }
        }
        Ok(false)
    }
}
#[async_trait]
impl SetFaultLight for AvalonMiner {
    async fn set_fault_light(&self, fault: bool) -> Result<bool> {
        let command = if fault { "1-1" } else { "1-0" };

        let data = self
            .rpc
            .send_command("ascset", false, Some(json!(["0", "led", command])))
            .await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_array()) {
            if let Some(msg) = status
                .first()
                .and_then(|s| s.get("Msg"))
                .and_then(|m| m.as_str())
            {
                return Ok(msg == "ASC 0 set OK");
            }
        }

        Err(anyhow!("Failed to set fault light to {}", command))
    }
}

#[async_trait]
impl SetPowerLimit for AvalonMiner {
    async fn set_power_limit(&self, limit: Power) -> Result<bool> {
        let data = self
            .rpc
            .send_command(
                "ascset",
                false,
                Some(json!(["0", "worklevel,set", limit.to_string()])),
            )
            .await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_array()) {
            if !status.is_empty() {
                if let Some(msg) = status[0].get("Msg").and_then(|m| m.as_str()) {
                    return Ok(msg == "ASC 0 set OK");
                }
            }
        }

        Err(anyhow!("Failed to set power limit"))
    }
}

impl GetDataLocations for AvalonMiner {
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        let version_cmd: MinerCommand = MinerCommand::RPC {
            command: "version",
            parameters: None,
        };
        let stats_cmd: MinerCommand = MinerCommand::RPC {
            command: "stats",
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

        match data_field {
            DataField::Mac => vec![(
                version_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/VERSION/0/MAC"),
                },
            )],
            DataField::ApiVersion => vec![(
                version_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/VERSION/0/API"),
                },
            )],
            DataField::FirmwareVersion => vec![(
                version_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/VERSION/0/CGMiner"),
                },
            )],
            DataField::Hashrate => vec![(
                devs_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/DEVS/0/MHS 1m"),
                },
            )],
            DataField::ExpectedHashrate => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/MM ID0:Summary/STATS/GHSmm"),
                },
            )],
            DataField::Hashboards => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/HBinfo"),
                },
            )],
            DataField::AverageTemperature => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/MM ID0:Summary/STATS/ITemp"),
                },
            )],
            DataField::WattageLimit => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/MM ID0:Summary/STATS/MPO"),
                },
            )],
            DataField::Wattage => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/MM ID0:Summary/STATS/WALLPOWER"),
                },
            )],
            DataField::Fans => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/MM ID0:Summary/STATS"),
                },
            )],
            DataField::LightFlashing => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/MM ID0:Summary/STATS/Led"),
                },
            )],
            DataField::Uptime => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS/0/Elapsed"),
                },
            )],
            DataField::Pools => vec![(
                pools_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/POOLS"),
                },
            )],
            _ => vec![],
        }
    }
}

impl GetIP for AvalonMiner {
    fn get_ip(&self) -> IpAddr {
        self.ip
    }
}

impl GetDeviceInfo for AvalonMiner {
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }
}

impl CollectData for AvalonMiner {
    fn get_collector(&self) -> DataCollector {
        DataCollector::new(self, &self.rpc)
    }
}

impl GetMAC for AvalonMiner {
    fn parse_mac(&self, data: &HashMap<DataField, Value>) -> Option<MacAddr> {
        data.extract::<String>(DataField::Mac).and_then(|raw| {
            let mut mac = raw.trim().to_lowercase();
            // compact 12-digit → colon-separated
            if mac.len() == 12 && !mac.contains(':') {
                let mut colon = String::with_capacity(17);
                for (i, byte) in mac.chars().enumerate() {
                    if i > 0 && i % 2 == 0 {
                        colon.push(':');
                    }
                    colon.push(byte);
                }
                mac = colon;
            }
            MacAddr::from_str(&mac).ok()
        })
    }
}

impl GetSerialNumber for AvalonMiner {}

impl GetHostname for AvalonMiner {}

impl GetApiVersion for AvalonMiner {
    fn parse_api_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::ApiVersion)
    }
}

impl GetFirmwareVersion for AvalonMiner {
    fn parse_firmware_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::FirmwareVersion)
    }
}

impl GetControlBoardVersion for AvalonMiner {}

impl GetHashboards for AvalonMiner {
    fn parse_hashboards(&self, data: &HashMap<DataField, Value>) -> Vec<BoardData> {
        let hw = &self.device_info.hardware;
        let board_cnt = hw.boards.unwrap_or(1) as usize;
        let chips_per = hw.chips.unwrap_or(0);

        let hb_info = match data.get(&DataField::Hashboards).and_then(|v| v.as_object()) {
            Some(v) => v,
            _ => return Vec::new(),
        };

        let summary = match data.get(&DataField::Fans) {
            Some(v) => v,
            _ => return Vec::new(),
        }; //some HB info is grouped with fan data.

        (0..board_cnt)
            .map(|idx| {
                let key = format!("HB{idx}");

                // per-board aggregates
                let intake = summary["ITemp"][idx]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(Temperature::from_celsius);
                let board_t = summary["HBITemp"][idx]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(Temperature::from_celsius);
                let hashrate = summary["MGHS"][idx]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|r| HashRate {
                        value: r,
                        unit: HashRateUnit::GigaHash,
                        algo: "SHA256".into(),
                    });

                // per-chip arrays
                let temps: Vec<String> = hb_info[&key]["PVT_T0"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str())
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default();
                let volts: Vec<String> = hb_info[&key]["PVT_V0"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str())
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default();
                let works: Vec<String> = hb_info[&key]["MW0"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str())
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default();

                let chips: Vec<ChipData> = temps
                    .iter()
                    .zip(volts.iter())
                    .zip(works.iter())
                    .enumerate()
                    .map(|(pos, ((t, v), w))| ChipData {
                        position: pos as u16,
                        temperature: t.parse::<f64>().ok().map(Temperature::from_celsius),
                        voltage: v.parse::<f64>().ok().map(Voltage::from_millivolts),
                        working: w.parse::<f64>().ok().map(|w| w > 0.0),
                        ..Default::default()
                    })
                    .collect();

                BoardData {
                    position: idx as u8,
                    expected_chips: Some(chips_per),
                    working_chips: Some(chips.len() as u16),
                    chips: chips.clone(),
                    intake_temperature: intake,
                    board_temperature: board_t,
                    hashrate,
                    active: Some(!chips.is_empty()),
                    ..Default::default()
                }
            })
            .collect()
    }
}

impl GetHashrate for AvalonMiner {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::MegaHash,
            algo: "SHA256".into(),
        })
    }
}

impl GetExpectedHashrate for AvalonMiner {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::ExpectedHashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::GigaHash,
            algo: "SHA256".into(),
        })
    }
}

impl GetFans for AvalonMiner {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        let stats = match data.get(&DataField::Fans) {
            Some(v) => v,
            _ => return Vec::new(),
        };

        let expected_fans = self.device_info.hardware.fans.unwrap_or(0) as usize;
        if expected_fans == 0 {
            return Vec::new();
        }

        (1..=expected_fans)
            .filter_map(|idx| {
                let key = format!("Fan{idx}");
                stats
                    .get(&key)
                    .and_then(|val| val.as_f64())
                    .map(|rpm| FanData {
                        position: idx as i16,
                        rpm: Some(AngularVelocity::from_rpm(rpm)),
                    })
            })
            .collect()
    }
}

impl GetPsuFans for AvalonMiner {}

impl GetAverageTemperature for AvalonMiner {
    fn parse_average_temperature(&self, data: &HashMap<DataField, Value>) -> Option<Temperature> {
        data.extract_map::<f64, _>(DataField::AverageTemperature, |f| {
            Temperature::from_celsius(f)
        })
    }
}

impl GetWattage for AvalonMiner {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<f64, _>(DataField::Wattage, Power::from_watts)
    }
}

impl GetWattageLimit for AvalonMiner {
    fn parse_wattage_limit(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<f64, _>(DataField::WattageLimit, Power::from_watts)
    }
}

impl GetLightFlashing for AvalonMiner {
    fn parse_light_flashing(&self, data: &HashMap<DataField, Value>) -> Option<bool> {
        data.extract::<bool>(DataField::LightFlashing)
    }
}

impl GetMessages for AvalonMiner {}

impl GetUptime for AvalonMiner {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.extract_map::<u64, _>(DataField::Uptime, Duration::from_secs)
    }
}

impl GetFluidTemperature for AvalonMiner {}
impl GetIsMining for AvalonMiner {}

impl GetPools for AvalonMiner {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolData> {
        data.get(&DataField::Pools)
            .and_then(|v| v.as_array())
            .map(|slice| slice.to_vec())
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(idx, pool)| PoolData {
                url: pool
                    .get("URL")
                    .and_then(|v| v.as_str())
                    .map(|x| PoolURL::from(x.to_owned())),
                user: pool.get("User").and_then(|v| v.as_str()).map(|s| s.into()),
                position: Some(idx as u16),
                alive: pool
                    .get("Status")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "Alive"),
                active: pool.get("Stratum Active").and_then(|v| v.as_bool()),
                accepted_shares: pool.get("Accepted").and_then(|v| v.as_u64()),
                rejected_shares: pool.get("Rejected").and_then(|v| v.as_u64()),
            })
            .collect()
    }
}
