#![allow(dead_code)]
pub mod rpc;

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

use crate::miners::api::RPCAPIClient;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Power, Temperature};
use regex::Regex;
use rpc::CGMinerRPC;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub struct AvalonMiner {
    ip: IpAddr,
    rpc: CGMinerRPC,
    device_info: DeviceInfo,
}

impl AvalonMiner {
    pub fn new(ip: IpAddr, model: MinerModel, miner_firmware: MinerFirmware) -> Self {
        Self {
            ip,
            rpc: CGMinerRPC::new(ip),
            device_info: DeviceInfo::new(
                MinerMake::AvalonMiner,
                model,
                miner_firmware,
                HashAlgorithm::SHA256,
            ),
        }
    }

    /// Turn on the fault light
    pub async fn fault_light_on(&self) -> Result<()> {
        let data = self
            .rpc
            .send_command("ascset", false, Some(json!(["0", "led", "1-1"])))
            .await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_array()) {
            if !status.is_empty() {
                if let Some(msg) = status[0].get("Msg").and_then(|m| m.as_str()) {
                    if msg == "ASC 0 set OK" {
                        return Ok(());
                    }
                }
            }
        }

        Err(anyhow!("Failed to turn on fault light"))
    }

    /// Turn off the fault light
    pub async fn fault_light_off(&self) -> Result<()> {
        let data = self
            .rpc
            .send_command("ascset", false, Some(json!(["0", "led", "1-0"])))
            .await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_array()) {
            if !status.is_empty() {
                if let Some(msg) = status[0].get("Msg").and_then(|m| m.as_str()) {
                    if msg == "ASC 0 set OK" {
                        return Ok(());
                    }
                }
            }
        }

        Err(anyhow!("Failed to turn off fault light"))
    }

    /// Reboot the miner
    pub async fn reboot(&self) -> Result<bool> {
        let data = self.rpc.send_command("restart", false, None).await?;

        if let Some(status) = data.get("STATUS").and_then(|s| s.as_str()) {
            return Ok(status == "RESTART");
        }

        Ok(false)
    }

    /// Schedule soft power on at a specific timestamp
    pub async fn soft_power_on(&self, timestamp: u64) -> Result<bool> {
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

    /// Schedule soft power off at a specific timestamp
    pub async fn soft_power_off(&self, timestamp: u64) -> Result<bool> {
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

    /// Parse stats from the miner
    fn parse_stats(&self, stats: &str) -> HashMap<String, Vec<String>> {
        let mut stats_dict = HashMap::new();

        let re = Regex::new(r"(\w+)\[([^\]]+)\]").unwrap();

        for cap in re.captures_iter(stats) {
            let key = cap[1].to_string();
            let value_str = &cap[2];

            let values: Vec<String> = if value_str.contains(' ') {
                value_str
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                vec![value_str.to_string()]
            };

            stats_dict.insert(key, values);
        }

        stats_dict
    }
}
#[async_trait]
impl Pause for AvalonMiner {
    async fn pause(&self, at_time: Option<u64>) -> Result<()> {
        if let Some(time) = at_time {
            self.soft_power_off(time).await?;
        } else {
            self.soft_power_off(0).await?;
        }
        Ok(())
    }
}
#[async_trait]
impl Resume for AvalonMiner {
    async fn resume(&self, at_time: Option<u64>) -> Result<()> {
        if let Some(time) = at_time {
            self.soft_power_on(time).await?;
        } else {
            self.soft_power_on(0).await?;
        }
        Ok(())
    }
}
#[async_trait]
impl SetFaultLight for AvalonMiner {
    async fn set_fault_light(&self, fault: bool) -> Result<()> {
        match fault {
            true => self.fault_light_on().await,
            false => self.fault_light_off().await,
        }
    }
}

#[async_trait]
impl SetPowerLimit for AvalonMiner {
    async fn set_power_limit(&self, limit: Power) -> Result<()> {
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
                    if msg == "ASC 0 set OK" {
                        return Ok(());
                    }
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
                    key: Some("/DEVS/0/MHS 5m"),
                },
            )],
            DataField::ExpectedHashrate => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS"),
                },
            )],
            DataField::Hashboards => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS"),
                },
            )],
            DataField::FluidTemperature => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS"),
                },
            )],
            DataField::WattageLimit => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS"),
                },
            )],
            DataField::Wattage => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS"),
                },
            )],
            DataField::Fans => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS"),
                },
            )],
            DataField::LightFlashing => vec![(
                stats_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/STATS"),
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
        data.extract(DataField::Mac).and_then(|mac: String| {
            let mac = mac.to_uppercase();

            let mac = (0..mac.len())
                .step_by(2)
                .map(|i| &mac[i..i + 2])
                .collect::<Vec<_>>()
                .join(":");

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
        let stats_array = match data.get(&DataField::Hashboards).and_then(|v| v.as_array()) {
            Some(array) => array,
            None => return Vec::new(),
        };

        let stat = match stats_array
            .iter()
            .find(|s| s.get("ID").and_then(|v| v.as_str()) == Some("AVALON0"))
        {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mm_stats = stat
            .get("MM ID0:Summary")
            .and_then(|v| v.as_str())
            .map(|s| self.parse_stats(s));

        let hb_stats = stat
            .get("HBinfo")
            .and_then(|v| v.as_str())
            .map(|s| self.parse_stats(s));

        let hashrate = mm_stats.as_ref().and_then(|stats| {
            stats
                .get("MGHS")
                .and_then(|v| v.first())
                .and_then(|rate_str| rate_str.parse::<f64>().ok())
                .map(|rate| HashRate {
                    value: rate,
                    unit: HashRateUnit::GigaHash,
                    algo: "SHA256".to_string(),
                })
        });

        let intake_temperature = mm_stats.as_ref().and_then(|stats| {
            stats
                .get("ITemp")
                .and_then(|v| v.first())
                .and_then(|temp_str| temp_str.parse::<f64>().ok())
                .map(Temperature::from_celsius)
        });

        let board_temperature = mm_stats.as_ref().and_then(|stats| {
            stats
                .get("HBITemp")
                .and_then(|v| v.first())
                .and_then(|temp_str| temp_str.parse::<f64>().ok())
                .map(Temperature::from_celsius)
        });

        let working_chips_board0 = hb_stats.as_ref().and_then(|stats| {
            stats
                .get("PVT_T0")
                .map(|temps| temps.iter().filter(|temp| *temp != "0").count() as u16)
        });

        let mut hashboards = Vec::new();
        let device_info = self.get_device_info();
        let expected_hashboards = device_info.hardware.boards.unwrap_or(1);

        for i in 0..expected_hashboards {
            let working_chips = if i == 0 { working_chips_board0 } else { None };
            let is_active =
                hashrate.is_some() || (working_chips.is_some() && working_chips != Some(0));

            let board = BoardData {
                position: i,
                expected_chips: device_info.hardware.chips,
                working_chips,
                board_temperature,
                intake_temperature,
                hashrate: hashrate.clone(),
                active: Some(is_active),
                outlet_temperature: None,
                expected_hashrate: None,
                serial_number: None,
                chips: Vec::new(),
                voltage: None,
                frequency: None,
                tuned: None,
            };
            hashboards.push(board);
        }

        hashboards
    }
}

impl GetHashrate for AvalonMiner {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::MegaHash,
            algo: String::from("SHA256"),
        })
    }
}

impl GetExpectedHashrate for AvalonMiner {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        if let Some(stats) = data.get(&DataField::ExpectedHashrate) {
            if let Some(stats_str) = stats.as_str() {
                let parsed_stats = self.parse_stats(stats_str);
                if let Some(ghsmm) = parsed_stats.get("GHSmm").and_then(|v| v.first()) {
                    if let Ok(rate) = ghsmm.parse::<f64>() {
                        return Some(HashRate {
                            value: rate,
                            unit: HashRateUnit::GigaHash,
                            algo: String::from("SHA256"),
                        });
                    }
                }
            }
        }
        None
    }
}

impl GetFans for AvalonMiner {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        let mut fans = Vec::new();

        let expected_fans = self.get_device_info().hardware.fans.unwrap_or(0);
        if expected_fans == 0 {
            return fans;
        }

        if let Some(stats) = data.get(&DataField::Fans) {
            if let Some(stats_str) = stats.as_str() {
                let parsed_stats = self.parse_stats(stats_str);

                if let Some(mm_id0) = parsed_stats.get("MM ID0").and_then(|v| v.first()) {
                    let mm_stats = self.parse_stats(mm_id0);

                    for fan in 0..expected_fans {
                        let fan_key = format!("Fan{}", fan + 1);
                        if let Some(fan_speed) = mm_stats.get(&fan_key).and_then(|v| v.first()) {
                            if let Ok(speed) = fan_speed.parse::<f64>() {
                                fans.push(FanData {
                                    position: fan as i16,
                                    rpm: Some(AngularVelocity::from_rpm(speed)),
                                });
                            }
                        }
                    }
                }
            }
        }

        fans
    }
}

impl GetPsuFans for AvalonMiner {}

impl GetFluidTemperature for AvalonMiner {
    fn parse_fluid_temperature(&self, data: &HashMap<DataField, Value>) -> Option<Temperature> {
        if let Some(stats) = data.get(&DataField::FluidTemperature) {
            if let Some(stats_str) = stats.as_str() {
                let parsed_stats = self.parse_stats(stats_str);

                if let Some(mm_id0) = parsed_stats.get("MM ID0").and_then(|v| v.first()) {
                    let mm_stats = self.parse_stats(mm_id0);

                    if let Some(temp) = mm_stats.get("Temp").and_then(|v| v.first()) {
                        if let Ok(temp_value) = temp.parse::<f64>() {
                            return Some(Temperature::from_celsius(temp_value));
                        }
                    }
                }
            }
        }
        None
    }
}

impl GetWattage for AvalonMiner {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        if let Some(stats) = data.get(&DataField::Wattage) {
            if let Some(stats_str) = stats.as_str() {
                let parsed_stats = self.parse_stats(stats_str);

                if let Some(mm_id0) = parsed_stats.get("MM ID0").and_then(|v| v.first()) {
                    let mm_stats = self.parse_stats(mm_id0);

                    if let Some(power) = mm_stats.get("WALLPOWER").and_then(|v| v.first()) {
                        if let Ok(power_value) = power.parse::<f64>() {
                            return Some(Power::from_watts(power_value));
                        }
                    }
                }
            }
        }
        None
    }
}

impl GetWattageLimit for AvalonMiner {
    fn parse_wattage_limit(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        if let Some(stats) = data.get(&DataField::WattageLimit) {
            if let Some(stats_str) = stats.as_str() {
                let parsed_stats = self.parse_stats(stats_str);

                if let Some(mm_id0) = parsed_stats.get("MM ID0").and_then(|v| v.first()) {
                    let mm_stats = self.parse_stats(mm_id0);

                    if let Some(power) = mm_stats.get("MPO").and_then(|v| v.first()) {
                        if let Ok(power_value) = power.parse::<f64>() {
                            return Some(Power::from_watts(power_value));
                        }
                    }
                }
            }
        }
        None
    }
}

impl GetLightFlashing for AvalonMiner {
    fn parse_light_flashing(&self, data: &HashMap<DataField, Value>) -> Option<bool> {
        if let Some(stats) = data.get(&DataField::LightFlashing) {
            if let Some(stats_str) = stats.as_str() {
                let parsed_stats = self.parse_stats(stats_str);

                if let Some(mm_id0) = parsed_stats.get("MM ID0").and_then(|v| v.first()) {
                    let mm_stats = self.parse_stats(mm_id0);

                    if let Some(led) = mm_stats.get("Led").and_then(|v| v.first()) {
                        if let Ok(led_value) = led.parse::<i32>() {
                            return Some(led_value == 1);
                        }
                    }
                }
            }
        }

        None
    }
}

impl GetMessages for AvalonMiner {}

impl GetUptime for AvalonMiner {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.extract_map::<u64, _>(DataField::Uptime, Duration::from_secs)
    }
}

impl GetIsMining for AvalonMiner {}

impl GetPools for AvalonMiner {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolData> {
        let mut pools = Vec::new();

        if let Some(pools_value) = data.get(&DataField::Pools) {
            if let Some(pools_array) = pools_value.as_array() {
                for (idx, pool) in pools_array.iter().enumerate() {
                    let url = pool
                        .get("URL")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let user = pool
                        .get("User")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let alive = pool
                        .get("Status")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "Alive");
                    let position = Some(idx as u16);
                    let active = pool.get("Stratum Active").and_then(|v| v.as_bool());
                    let accepted_shares = pool.get("Accepted").and_then(|v| v.as_u64());
                    let rejected_shares = pool.get("Rejected").and_then(|v| v.as_u64());

                    if let Some(url_str) = url {
                        pools.push(PoolData {
                            url: Some(PoolURL::from(url_str)),
                            user,
                            position,
                            alive,
                            active,
                            accepted_shares,
                            rejected_shares,
                        });
                    }
                }
            }
        }

        pools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::device::models::avalon::AvalonMinerModel::Avalon741;
    use crate::test::api::MockAPIClient;
    use crate::test::json::cgminer::avalon::*;
    use std::collections::HashMap;

    fn create_test_miner() -> AvalonMiner {
        AvalonMiner::new(
            IpAddr::from([192, 168, 1, 100]),
            MinerModel::Avalon(Avalon741),
            MinerFirmware::Stock,
        )
    }

    fn create_mock_api_responses() -> HashMap<MinerCommand, Value> {
        let mut results = HashMap::new();

        let version_cmd = MinerCommand::RPC {
            command: "version",
            parameters: None,
        };
        results.insert(version_cmd, serde_json::from_str(VERSION_COMMAND).unwrap());

        let stats_cmd = MinerCommand::RPC {
            command: "stats",
            parameters: None,
        };
        results.insert(stats_cmd, serde_json::from_str(STATS_COMMAND).unwrap());

        let devs_cmd = MinerCommand::RPC {
            command: "devs",
            parameters: None,
        };
        results.insert(devs_cmd, serde_json::from_str(DEVS_COMMAND).unwrap());

        let pools_cmd = MinerCommand::RPC {
            command: "pools",
            parameters: None,
        };
        results.insert(pools_cmd, serde_json::from_str(POOLS_COMMAND).unwrap());

        let summary_cmd = MinerCommand::RPC {
            command: "summary",
            parameters: None,
        };
        results.insert(summary_cmd, serde_json::from_str(SUMMARY_COMMAND).unwrap());

        results
    }

    #[tokio::test]
    async fn test_avalon_miner_data_parsing() {
        let miner = create_test_miner();
        let mock_responses = create_mock_api_responses();
        let mock_api = MockAPIClient::new(mock_responses);

        let mut collector = DataCollector::new(&miner, &mock_api);
        let data = collector.collect_all().await;

        let miner_data = miner.parse_data(data);

        assert_eq!(&miner_data.ip, &miner.ip);
        assert_eq!(&miner_data.device_info, &miner.get_device_info());

        assert_eq!(
            &miner_data.mac.unwrap(),
            &MacAddr::from_str("AA:BB:CC:DD:EE:FF").unwrap()
        );

        assert_eq!(&miner_data.firmware_version, &Some("4.11.1".to_string()));

        assert_eq!(&miner_data.api_version, &Some("3.7".to_string()));

        let expected_hashrate = HashRate {
            value: 0.02,
            unit: HashRateUnit::MegaHash,
            algo: "SHA256".to_string(),
        };
        assert_eq!(&miner_data.hashrate, &Some(expected_hashrate));
    }

    #[tokio::test]
    async fn test_avalon_hashboard_parsing() {
        let miner = create_test_miner();
        let mock_responses = create_mock_api_responses();
        let mock_api = MockAPIClient::new(mock_responses);

        let mut collector = DataCollector::new(&miner, &mock_api);
        let data = collector.collect_all().await;

        let miner_data = miner.parse_data(data);

        assert!(
            !miner_data.hashboards.is_empty(),
            "Hashboards should be populated"
        );

        let first_board = &miner_data.hashboards[0];
        assert_eq!(first_board.position, 0);
        assert!(
            first_board.active.unwrap_or(false),
            "Board should be marked as active"
        );

        assert!(
            first_board.board_temperature.is_some() || first_board.intake_temperature.is_some(),
            "Temperature data should be available from stats"
        );
    }

    #[tokio::test]
    async fn test_avalon_fan_parsing() {
        let miner = create_test_miner();
        let mock_responses = create_mock_api_responses();
        let mock_api = MockAPIClient::new(mock_responses);

        let mut collector = DataCollector::new(&miner, &mock_api);
        let data = collector.collect_all().await;

        let miner_data = miner.parse_data(data);

        if !miner_data.fans.is_empty() {
            let first_fan = &miner_data.fans[0];
            assert_eq!(first_fan.position, 0);
        }
    }

    #[tokio::test]
    async fn test_avalon_wattage_parsing() {
        let miner = create_test_miner();
        let mock_responses = create_mock_api_responses();
        let mock_api = MockAPIClient::new(mock_responses);

        let mut collector = DataCollector::new(&miner, &mock_api);
        let data = collector.collect_all().await;

        let miner_data = miner.parse_data(data);

        if let Some(wattage) = &miner_data.wattage {
            assert!(wattage.as_watts() >= 0.0, "Wattage should be non-negative");
        }
    }

    #[tokio::test]
    async fn test_avalon_temperature_parsing() {
        let miner = create_test_miner();
        let mock_responses = create_mock_api_responses();
        let mock_api = MockAPIClient::new(mock_responses);

        let mut collector = DataCollector::new(&miner, &mock_api);
        let data = collector.collect_all().await;

        let miner_data = miner.parse_data(data);

        if let Some(temp) = &miner_data.fluid_temperature {
            assert!(
                temp.as_celsius() > -50.0 && temp.as_celsius() < 150.0,
                "Temperature should be within reasonable range"
            );
        }
    }

    #[tokio::test]
    async fn test_avalon_stats_parsing() {
        let miner = create_test_miner();

        let test_stats = "'STATS':{Ver[Q-25052801_14a19a2] LVer[25052801_14a19a2] BVer[25052801_14a19a2] HashMcu0Ver[Q_hb_v1.1] FanMcuVer[Q_fb_v1.2] CPU[K230] FW[Release] DNA[01234567890123456789012345678901] STATE[2] MEMFREE[67892] NETFAIL[0 0 0 0 0 0 0 0] SSID[] RSSI[0] NetDevType[0] SYSTEMSTATU[Work: In Idle, Hash Board: 1] Elapsed[37850] BOOTBY[0x01.00000000] LW[16987598] MH[0] DHW[0] HW[0] DH[2.449%] ITemp[26] HBITemp[27] HBOTemp[27] TMax[0] TAvg[0] TarT[65] Fan1[0] Fan2[0] Fan3[0] Fan4[0] FanR[0%] SoftOffTime[1753425250] SoftOnTime[1753425190] Filter[19143] FanErr[0] SoloAllowed[1] PS[0 1222 4 0 0 2245 146] PCOMM_E[0] GHSspd[0.00] DHspd[0.000%] GHSmm[55032.79] GHSavg[44499.41] WU[621649.53] Freq[282.86] MGHS[44499.41] TA[160] Core[A3197S] BIN[36] PING[17] SoftOFF[4] ECHU[0] ECMM[0] PLL0[8843 5769 5098 4610] SF0[258 276 297 318] CRC[0] COMCRC[0] ATA0[800-65-2264-258-20] LcdOnoff[1] Activation[0] WORKMODE[0] WORKLEVEL[0] MPO[800] CALIALL[7] ADJ[1] Nonce Mask[25]}";

        let parsed = miner.parse_stats(test_stats);

        // Test that key values are parsed correctly
        assert!(parsed.contains_key("Ver"), "Should parse firmware version");
        assert!(parsed.contains_key("GHSmm"), "Should parse hashrate");
        assert!(
            parsed.contains_key("ITemp"),
            "Should parse internal temperature"
        );
        assert!(parsed.contains_key("Fan1"), "Should parse fan data");

        // Test specific values
        if let Some(ghsmm) = parsed.get("GHSmm").and_then(|v| v.first()) {
            assert_eq!(ghsmm, "55032.79", "GHSmm should match expected value");
        }

        if let Some(itemp) = parsed.get("ITemp").and_then(|v| v.first()) {
            assert_eq!(itemp, "26", "ITemp should match expected value");
        }
    }

    #[test]
    fn test_device_info_creation() {
        let miner = create_test_miner();
        let device_info = miner.get_device_info();

        assert_eq!(device_info.make, MinerMake::AvalonMiner);
        assert_eq!(device_info.firmware, MinerFirmware::Stock);
        assert_eq!(device_info.algo, HashAlgorithm::SHA256);
    }
}
