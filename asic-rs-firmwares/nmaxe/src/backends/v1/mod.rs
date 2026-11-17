use std::{collections::HashMap, net::IpAddr, time::Duration};

use self::web::NMAxeWebAPI;
use crate::firmware::NMAxeFirmware;
use asic_rs_core::{
    config::{
        collector::{ConfigCollector, ConfigExtractor, ConfigField, ConfigLocation},
        pools::{PoolConfig, PoolGroupConfig},
    },
    data::{
        board::BoardData,
        collector::{
            DataCollector, DataExtensions, DataExtractor, DataField, DataLocation, get_by_pointer,
        },
        command::MinerCommand,
        device::{DeviceInfo, HashAlgorithm},
        fan::FanData,
        hashrate::{HashRate, HashRateUnit},
        operating_state::OperatingState,
        pool::{PoolData, PoolGroupData, PoolURL},
        share::parse_share_difficulty,
    },
    traits::{miner::*, model::MinerModel},
};
use async_trait::async_trait;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};
use reqwest::Method;
use serde_json::{Value, json};

pub(in crate::backends) mod web;

const MINING_SETTINGS: MinerCommand = MinerCommand::WebAPI {
    command: "setting/mining",
    parameters: None,
};

/// Backend for the nested system/info API documented by ESP-Miner-NMAxe.
/// <https://github.com/NMminer1024/ESP-Miner-NMAxe/blob/master/docs/API.md>
#[derive(Debug)]
pub struct NMAxeV1 {
    ip: IpAddr,
    web: NMAxeWebAPI,
    device_info: DeviceInfo,
}

impl NMAxeV1 {
    pub fn new(ip: IpAddr, model: impl MinerModel) -> Self {
        Self {
            ip,
            web: NMAxeWebAPI::new(ip, 80),
            device_info: DeviceInfo::new(model, NMAxeFirmware::default(), HashAlgorithm::SHA256),
        }
    }
}

#[async_trait]
impl APIClient for NMAxeV1 {
    async fn get_api_result(&self, command: &MinerCommand) -> anyhow::Result<Value> {
        self.web.get_api_result(command).await
    }
}

impl GetConfigsLocations for NMAxeV1 {
    fn get_configs_locations(&self, field: ConfigField) -> Vec<ConfigLocation> {
        match field {
            ConfigField::Pools => vec![(
                MINING_SETTINGS,
                ConfigExtractor {
                    func: get_by_pointer,
                    key: Some("/stratum"),
                    tag: None,
                },
            )],
            _ => vec![],
        }
    }
}
impl CollectConfigs for NMAxeV1 {
    fn get_config_collector(&self) -> ConfigCollector<'_> {
        ConfigCollector::new(self)
    }
}

impl GetDataLocations for NMAxeV1 {
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        const WEB_SYSTEM_INFO: MinerCommand = MinerCommand::WebAPI {
            command: "system/info",
            parameters: None,
        };

        match data_field {
            DataField::Hostname => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/identity/hostName"),
                    tag: None,
                },
            )],
            DataField::FirmwareVersion => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/identity/fwVersion"),
                    tag: None,
                },
            )],
            DataField::Hashboards => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::Hashrate => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/hashRate"),
                    tag: None,
                },
            )],
            DataField::ExpectedHashrate => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::Fans => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/fans"),
                    tag: None,
                },
            )],
            DataField::Wattage => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/power/power"),
                    tag: None,
                },
            )],
            DataField::Uptime => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/uptimeSeconds"),
                    tag: None,
                },
            )],
            DataField::Pools => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::BestShare => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/bestDiffEver"),
                    tag: None,
                },
            )],
            DataField::SessionBestShare => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/bestDiffSession"),
                    tag: None,
                },
            )],
            DataField::OperatingState => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/state"),
                    tag: None,
                },
            )],
            DataField::IsMining => vec![(
                WEB_SYSTEM_INFO,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner"),
                    tag: None,
                },
            )],
            _ => vec![],
        }
    }
}

impl CollectData for NMAxeV1 {
    fn get_collector(&self) -> DataCollector<'_> {
        DataCollector::new(self)
    }
}
impl GetIP for NMAxeV1 {
    fn get_ip(&self) -> IpAddr {
        self.ip
    }
}
impl GetDeviceInfo for NMAxeV1 {
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }
}
impl GetMAC for NMAxeV1 {}
impl GetSerialNumber for NMAxeV1 {}
impl GetApiVersion for NMAxeV1 {}
impl GetControlBoardVersion for NMAxeV1 {}
impl GetHostname for NMAxeV1 {
    fn parse_hostname(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract(DataField::Hostname)
    }
}
impl GetFirmwareVersion for NMAxeV1 {
    fn parse_firmware_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract(DataField::FirmwareVersion)
    }
}
impl GetHashrate for NMAxeV1 {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map(DataField::Hashrate, |value| {
            HashRate {
                value,
                unit: HashRateUnit::GigaHash,
                algo: self.device_info.algo,
            }
            .as_default_unit()
        })
    }
}
impl GetExpectedHashrate for NMAxeV1 {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        let info = data.get(&DataField::ExpectedHashrate)?;
        // smallCoreCnt is already the total across all ASICs.
        Some(
            HashRate {
                value: info.pointer("/asic/smallCoreCnt")?.as_f64()?
                    * info.pointer("/asic/freqReq")?.as_f64()?
                    / 1000.0,
                unit: HashRateUnit::GigaHash,
                algo: self.device_info.algo,
            }
            .as_default_unit(),
        )
    }
}
impl GetHashboards for NMAxeV1 {
    fn parse_hashboards(&self, data: &HashMap<DataField, Value>) -> Vec<BoardData> {
        let Some(info) = data.get(&DataField::Hashboards) else {
            return vec![];
        };
        let mut board = BoardData::new(0, self.device_info.hardware.chips_for_board(0));
        board.working_chips = info
            .pointer("/asic/count")
            .and_then(Value::as_u64)
            .and_then(|n| u16::try_from(n).ok());
        board.hashrate = info
            .pointer("/miner/hashRate")
            .and_then(Value::as_f64)
            .map(|value| {
                HashRate {
                    value,
                    unit: HashRateUnit::GigaHash,
                    algo: self.device_info.algo,
                }
                .as_default_unit()
            });
        // smallCoreCnt is already the total across all ASICs.
        board.expected_hashrate = info
            .pointer("/asic/smallCoreCnt")
            .and_then(Value::as_f64)
            .zip(info.pointer("/asic/freqReq").and_then(Value::as_f64))
            .map(|(core_count, frequency)| {
                HashRate {
                    value: core_count * frequency / 1000.0,
                    unit: HashRateUnit::GigaHash,
                    algo: self.device_info.algo,
                }
                .as_default_unit()
            });
        board.board_temperature = info
            .pointer("/temps/vcore")
            .and_then(Value::as_f64)
            .map(Temperature::from_celsius);
        let chip_temp = info
            .pointer("/temps/asic")
            .and_then(Value::as_f64)
            .map(Temperature::from_celsius);
        board.inlet_chip_temperature = chip_temp;
        board.outlet_chip_temperature = chip_temp;
        board.voltage = info
            .pointer("/asic/vcoreReal")
            .and_then(Value::as_f64)
            .map(Voltage::from_millivolts);
        board.frequency = info
            .pointer("/asic/freqReq")
            .and_then(Value::as_f64)
            .map(Frequency::from_megahertz);
        board.active = info
            .pointer("/miner/state")
            .and_then(Value::as_str)
            .map(|state| state == "running");
        vec![board]
    }
}
impl GetFans for NMAxeV1 {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        data.get(&DataField::Fans)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|fan| {
                Some(FanData {
                    position: fan.get("id")?.as_u64()?.try_into().ok()?,
                    rpm: fan
                        .get("rpm")
                        .and_then(Value::as_f64)
                        .map(AngularVelocity::from_rpm),
                })
            })
            .collect()
    }
}
impl GetWattage for NMAxeV1 {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map(DataField::Wattage, Power::from_watts)
    }
}
impl GetUptime for NMAxeV1 {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.extract_map(DataField::Uptime, Duration::from_secs)
    }
}
impl GetBestShare for NMAxeV1 {
    fn parse_best_share(&self, data: &HashMap<DataField, Value>) -> Option<f64> {
        data.get(&DataField::BestShare)
            .and_then(parse_share_difficulty)
    }
}
impl GetSessionBestShare for NMAxeV1 {
    fn parse_session_best_share(&self, data: &HashMap<DataField, Value>) -> Option<f64> {
        data.get(&DataField::SessionBestShare)
            .and_then(parse_share_difficulty)
    }
}
impl GetOperatingState for NMAxeV1 {
    fn parse_operating_state(&self, data: &HashMap<DataField, Value>) -> Option<OperatingState> {
        let state = data.get(&DataField::OperatingState)?.as_str()?;
        match state {
            "running" => Some(OperatingState::Mining {}),
            "pausing" => Some(OperatingState::Stopping {}),
            "resuming" => Some(OperatingState::Starting {}),
            _ => OperatingState::from_label(state),
        }
    }
}
impl GetIsMining for NMAxeV1 {
    fn parse_is_mining(&self, data: &HashMap<DataField, Value>) -> bool {
        data.get(&DataField::IsMining)
            .and_then(|miner| miner.get("state"))
            .and_then(Value::as_str)
            == Some("running")
    }
}
impl GetPools for NMAxeV1 {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolGroupData> {
        let Some(info) = data.get(&DataField::Pools) else {
            return vec![];
        };
        let Some(url) = info
            .pointer("/stratum/url")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        else {
            return vec![];
        };
        vec![PoolGroupData {
            name: "default".into(),
            quota: 1,
            pools: vec![PoolData {
                position: Some(0),
                url: Some(PoolURL::from(url.to_string())),
                user: info
                    .pointer("/stratum/user")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                accepted_shares: info.pointer("/miner/sAccepted").and_then(Value::as_u64),
                rejected_shares: info.pointer("/miner/sRejected").and_then(Value::as_u64),
                active: Some(true),
                alive: None,
                last_share_time: None,
            }],
        }]
    }
}

#[async_trait]
impl SupportsPoolsConfig for NMAxeV1 {
    fn supports_pools_config(&self) -> bool {
        true
    }
    fn parse_pools_config(
        &self,
        data: &HashMap<ConfigField, Value>,
    ) -> anyhow::Result<Vec<PoolGroupConfig>> {
        let stratum = data
            .get(&ConfigField::Pools)
            .ok_or_else(|| anyhow::anyhow!("Missing mining settings"))?;
        let mut pools = Vec::new();
        for name in ["primary", "fallback"] {
            if let Some(pool) = stratum.get(name) {
                let url = pool.get("url").and_then(Value::as_str).unwrap_or_default();
                if url.is_empty() {
                    continue;
                }
                pools.push(PoolConfig {
                    url: PoolURL::from(url.to_string()),
                    username: pool
                        .get("user")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    password: pool
                        .get("pwd")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                });
            }
        }
        Ok(vec![PoolGroupConfig {
            name: "default".into(),
            quota: 1,
            pools,
        }])
    }
    async fn set_pools_config(&self, config: Vec<PoolGroupConfig>) -> anyhow::Result<bool> {
        anyhow::ensure!(
            config.len() == 1 && (1..=2).contains(&config[0].pools.len()),
            "NMAxe requires one pool group with one or two pools"
        );
        let pools = &config[0].pools;
        let endpoint = |pool: &PoolConfig| json!({ "url": pool.url.to_string(), "user": pool.username, "pwd": pool.password });
        // Firmware validates both URLs and cannot clear the fallback with an empty URL.
        // Use the primary for both slots when only one endpoint is requested.
        let fallback = pools
            .get(1)
            .map(endpoint)
            .unwrap_or_else(|| endpoint(&pools[0]));
        self.web
            .send_command(
                "setting/mining",
                false,
                Some(
                    json!({ "stratum": { "primary": endpoint(&pools[0]), "fallback": fallback } }),
                ),
                Method::PATCH,
            )
            .await?;
        Ok(true)
    }
}
#[async_trait]
impl Restart for NMAxeV1 {
    fn supports_restart(&self) -> bool {
        true
    }
    async fn restart(&self) -> anyhow::Result<bool> {
        self.web
            .send_command("system/restart", false, None, Method::POST)
            .await?;
        Ok(true)
    }
}
#[async_trait]
impl Pause for NMAxeV1 {
    fn supports_pause(&self) -> bool {
        true
    }
    async fn pause(&self, at_time: Option<Duration>) -> anyhow::Result<bool> {
        anyhow::ensure!(at_time.is_none(), "NMAxe does not support scheduled pause");
        self.web
            .send_command(
                "mining/state",
                false,
                Some(json!({ "paused": true })),
                Method::PATCH,
            )
            .await?;
        Ok(true)
    }
}
#[async_trait]
impl Resume for NMAxeV1 {
    fn supports_resume(&self) -> bool {
        true
    }
    async fn resume(&self, at_time: Option<Duration>) -> anyhow::Result<bool> {
        anyhow::ensure!(at_time.is_none(), "NMAxe does not support scheduled resume");
        self.web
            .send_command(
                "mining/state",
                false,
                Some(json!({ "paused": false })),
                Method::PATCH,
            )
            .await?;
        Ok(true)
    }
}
impl GetPsuFans for NMAxeV1 {}
impl GetPsus for NMAxeV1 {}
impl GetFluidTemperature for NMAxeV1 {}
impl GetTuningTarget for NMAxeV1 {}
impl GetScaledTuningTarget for NMAxeV1 {}
impl GetTuningCapabilities for NMAxeV1 {}
impl GetLightFlashing for NMAxeV1 {}
impl GetMessages for NMAxeV1 {}
impl GetDevFeeConnected for NMAxeV1 {}
impl SupportsTimezoneConfig for NMAxeV1 {}
#[async_trait]
impl SetFaultLight for NMAxeV1 {
    fn supports_set_fault_light(&self) -> bool {
        false
    }
}

#[async_trait]
impl SetPowerLimit for NMAxeV1 {
    fn supports_set_power_limit(&self) -> bool {
        false
    }
}

impl ChangePassword for NMAxeV1 {
    fn supports_change_password(&self) -> bool {
        false
    }
}

impl ReadLogs for NMAxeV1 {
    fn supports_read_logs(&self) -> bool {
        false
    }
}

impl FactoryReset for NMAxeV1 {
    fn supports_factory_reset(&self) -> bool {
        false
    }
}

impl RestartMining for NMAxeV1 {}
impl RestoreStockOs for NMAxeV1 {}

#[async_trait]
impl SupportsScalingConfig for NMAxeV1 {
    fn supports_scaling_config(&self) -> bool {
        false
    }
}

#[async_trait]
impl UpgradeFirmware for NMAxeV1 {
    fn supports_upgrade_firmware(&self) -> bool {
        false
    }
}

impl HasAuth for NMAxeV1 {}
impl HasDefaultAuth for NMAxeV1 {}

impl Validate for NMAxeV1 {
    type Firmware = NMAxeFirmware;
}

#[async_trait]
impl SupportsTuningConfig for NMAxeV1 {
    fn supports_tuning_config(&self) -> bool {
        false
    }
}

#[async_trait]
impl SupportsFanConfig for NMAxeV1 {
    fn supports_fan_config(&self) -> bool {
        false
    }
}

impl SupportsTemperatureConfig for NMAxeV1 {}
impl GetTuningPercent for NMAxeV1 {}
impl SetHashboardsEnabled for NMAxeV1 {}

impl SetTuningPercent for NMAxeV1 {}
impl SupportsPresets for NMAxeV1 {}

#[cfg(test)]
mod tests {
    use super::*;
    use asic_rs_makes_nmaxe::models::NMAxeModel;

    fn miner() -> NMAxeV1 {
        NMAxeV1::new(IpAddr::from([127, 0, 0, 1]), NMAxeModel::NMQAxePlusPlus)
    }

    fn info() -> Value {
        json!({
            "identity": {"fwVersion": "v2.9.31", "hwModel": "NMQAxe++", "hostName": "nmaxe-001"},
            "power": {"power": 65.5, "vbus": 12000, "ibus": 5458},
            "temps": {"asic": 62.7, "vcore": 45.2},
            "asic": {"count": 4, "model": "BM1370", "vcoreReq": 1200, "vcoreReal": 1198, "freqReq": 490, "smallCoreCnt": 2688},
            "miner": {"state": "running", "paused": false, "hashRate": 4800.5, "bestDiffEver": "1.23M", "bestDiffSession": "456.7K", "sAccepted": 1024, "sRejected": 3, "uptimeSeconds": 86400},
            "fans": [{"id": 0, "speed": 60, "rpm": 2800}, {"id": 1, "speed": 80, "rpm": 4200}],
            "stratum": {"url": "stratum+tcp://pool.example.com:3333", "user": "account.worker", "pwd": "secret"}
        })
    }

    fn extracted(miner: &NMAxeV1, fields: &[DataField]) -> HashMap<DataField, Value> {
        let info = info();
        fields
            .iter()
            .filter_map(|field| {
                let (_, extractor) = miner.get_locations(*field).into_iter().next()?;
                Some((*field, (extractor.func)(&info, extractor.key)?.clone()))
            })
            .collect()
    }

    #[test]
    fn nested_telemetry_and_units() -> anyhow::Result<()> {
        let miner = miner();
        let data = extracted(
            &miner,
            &[
                DataField::Hostname,
                DataField::FirmwareVersion,
                DataField::Hashrate,
                DataField::ExpectedHashrate,
                DataField::Hashboards,
                DataField::Fans,
                DataField::Wattage,
                DataField::Uptime,
                DataField::BestShare,
                DataField::SessionBestShare,
                DataField::Pools,
                DataField::OperatingState,
                DataField::IsMining,
            ],
        );
        assert_eq!(miner.parse_hostname(&data).as_deref(), Some("nmaxe-001"));
        assert_eq!(
            miner.parse_firmware_version(&data).as_deref(),
            Some("v2.9.31")
        );
        assert_eq!(miner.parse_hashrate(&data).map(|h| h.value), Some(4.8005));
        assert_eq!(
            miner.parse_expected_hashrate(&data).map(|h| h.value),
            Some(1.31712)
        );
        let boards = miner.parse_hashboards(&data);
        assert_eq!(boards.len(), 1);
        assert_eq!(boards[0].expected_chips, Some(4));
        assert_eq!(boards[0].working_chips, Some(4));
        assert!(
            boards[0]
                .board_temperature
                .is_some_and(|t| (t.as_celsius() - 45.2).abs() < 1e-9)
        );
        assert!(
            boards[0]
                .inlet_chip_temperature
                .is_some_and(|t| (t.as_celsius() - 62.7).abs() < 1e-9)
        );
        assert_eq!(boards[0].voltage.map(|v| v.as_volts()), Some(1.198));
        assert_eq!(miner.parse_fans(&data).len(), 2);
        assert_eq!(miner.parse_wattage(&data).map(|p| p.as_watts()), Some(65.5));
        assert_eq!(miner.parse_uptime(&data), Some(Duration::from_secs(86400)));
        assert_eq!(miner.parse_best_share(&data), Some(1_230_000.0));
        assert_eq!(miner.parse_session_best_share(&data), Some(456_700.0));
        let pools = miner.parse_pools(&data);
        assert_eq!(pools[0].pools[0].accepted_shares, Some(1024));
        assert_eq!(pools[0].pools[0].rejected_shares, Some(3));
        assert_eq!(
            pools[0].pools[0].url.as_ref().map(|u| u.host.as_str()),
            Some("pool.example.com")
        );
        assert_eq!(
            miner.parse_operating_state(&data),
            Some(OperatingState::Mining {})
        );
        assert!(miner.parse_is_mining(&data));
        Ok(())
    }

    #[test]
    fn nexus_reports_two_expected_chips_and_preserves_live_counts() {
        let miner = NMAxeV1::new(
            IpAddr::from([127, 0, 0, 1]),
            NMAxeModel::NMQAxePlusPlusNexus,
        );
        let mut info = info();
        info["identity"]["displayName"] = json!("NMQAxe++Nexus");
        info["asic"]["model"] = json!("BM1373");
        info["asic"]["count"] = json!(2);
        let mut data = HashMap::from([(DataField::Hashboards, info)]);
        let boards = miner.parse_hashboards(&data);
        assert_eq!(miner.get_device_info().hardware.total_chips(), Some(2));
        assert_eq!(boards[0].expected_chips, Some(2));
        assert_eq!(boards[0].working_chips, Some(2));
        if let Some(info) = data.get_mut(&DataField::Hashboards) {
            info["asic"]["count"] = json!(1);
        }
        let boards = miner.parse_hashboards(&data);
        assert_eq!(boards[0].expected_chips, Some(2));
        assert_eq!(boards[0].working_chips, Some(1));
    }

    #[test]
    fn missing_telemetry_and_runtime_states() {
        let miner = miner();
        let empty = HashMap::new();
        assert_eq!(miner.parse_expected_hashrate(&empty), None);
        assert!(miner.parse_hashboards(&empty).is_empty());
        assert!(miner.parse_fans(&empty).is_empty());
        assert!(miner.parse_pools(&empty).is_empty());
        assert_eq!(miner.parse_operating_state(&empty), None);
        for (label, state) in [
            ("paused", OperatingState::Paused {}),
            ("pausing", OperatingState::Stopping {}),
            ("resuming", OperatingState::Starting {}),
            ("error", OperatingState::Error {}),
            (
                "future",
                OperatingState::Unknown {
                    raw: "future".into(),
                },
            ),
        ] {
            let data = HashMap::from([
                (DataField::OperatingState, json!(label)),
                (DataField::IsMining, json!({"state": label})),
            ]);
            assert_eq!(miner.parse_operating_state(&data), Some(state));
            assert!(!miner.parse_is_mining(&data));
        }
    }

    #[test]
    fn pool_settings_preserve_passwords_and_fallback() -> anyhow::Result<()> {
        let settings = HashMap::from([(
            ConfigField::Pools,
            json!({
                "primary": {"url": "stratum+tcp://primary.example:3333", "user": "main.worker", "pwd": "primary-secret"},
                "fallback": {"url": "stratum+tcp://fallback.example:4444", "user": "backup.worker", "pwd": "fallback-secret"}
            }),
        )]);
        let groups = miner().parse_pools_config(&settings)?;
        assert_eq!(groups[0].pools.len(), 2);
        assert_eq!(groups[0].pools[0].password, "primary-secret");
        assert_eq!(groups[0].pools[1].password, "fallback-secret");
        Ok(())
    }
    async fn mock_http(
        status: u16,
        body: &'static str,
    ) -> anyhow::Result<(NMAxeV1, tokio::task::JoinHandle<anyhow::Result<String>>)> {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await?;
            let mut request = Vec::new();
            let mut byte = [0u8; 1];
            while !request.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte).await?;
                request.push(byte[0]);
            }
            let header = String::from_utf8(request.clone())?;
            let length: usize = header
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse().ok())
                        .flatten()
                })
                .unwrap_or(0);
            let mut payload = vec![0; length];
            socket.read_exact(&mut payload).await?;
            request.extend(payload);
            let response = format!(
                "HTTP/1.1 {status} Response\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await?;
            Ok(String::from_utf8(request)?)
        });
        let mut miner = miner();
        miner.web = NMAxeWebAPI::new(addr.ip(), addr.port());
        Ok((miner, task))
    }

    #[tokio::test]
    async fn runtime_commands_and_plain_text_restart() -> anyhow::Result<()> {
        let (miner, request) = mock_http(200, "System will restart shortly.").await?;
        assert!(miner.restart().await?);
        assert!(
            request
                .await??
                .starts_with("POST /api/system/restart HTTP/1.1")
        );
        for paused in [true, false] {
            let (miner, request) = mock_http(200, r#"{"status":"ok"}"#).await?;
            assert!(if paused {
                miner.pause(None).await?
            } else {
                miner.resume(None).await?
            });
            let request = request.await??;
            assert!(request.starts_with("PATCH /api/mining/state HTTP/1.1"));
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("content-type: application/json")
            );
            let (_, body) = request
                .split_once("\r\n\r\n")
                .ok_or_else(|| anyhow::anyhow!("Missing HTTP body"))?;
            assert_eq!(
                serde_json::from_str::<Value>(body)?,
                json!({"paused": paused})
            );
        }
        let (miner, request) = mock_http(409, r#"{"error":"benchmark running"}"#).await?;
        assert!(miner.pause(None).await.is_err());
        request.await??;
        assert!(miner.pause(Some(Duration::from_secs(60))).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn pool_write_uses_mining_settings_schema() -> anyhow::Result<()> {
        let (miner, request) = mock_http(200, r#"{"status":"ok"}"#).await?;
        let pool = PoolConfig {
            url: PoolURL::from("stratum+tcp://pool.example:3333".to_string()),
            username: "account.worker".into(),
            password: "secret".into(),
        };
        assert!(
            miner
                .set_pools_config(vec![PoolGroupConfig {
                    name: "default".into(),
                    quota: 1,
                    pools: vec![pool]
                }])
                .await?
        );
        let request = request.await??;
        assert!(request.starts_with("PATCH /api/setting/mining HTTP/1.1"));
        let (_, body) = request
            .split_once("\r\n\r\n")
            .ok_or_else(|| anyhow::anyhow!("Missing HTTP body"))?;
        let body: Value = serde_json::from_str(body)?;
        assert_eq!(
            body["stratum"]["primary"],
            json!({"url": "stratum+tcp://pool.example:3333", "user": "account.worker", "pwd": "secret"})
        );
        assert_eq!(body["stratum"]["fallback"], body["stratum"]["primary"]);
        assert!(miner.set_pools_config(vec![]).await.is_err());
        Ok(())
    }
}
