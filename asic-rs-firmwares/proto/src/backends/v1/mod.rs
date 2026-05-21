use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
    net::IpAddr,
    str::FromStr,
    time::Duration,
};

use anyhow::Context;
use asic_rs_core::{
    config::{
        collector::{
            ConfigCollector, ConfigExtractor, ConfigField, ConfigLocation,
            get_by_pointer as get_config_by_pointer,
        },
        fan::FanConfig,
        pools::{PoolConfig, PoolGroupConfig},
        scaling::ScalingConfig,
        tuning::TuningConfig,
    },
    data::{
        board::{BoardData, ChipData, MinerControlBoard},
        collector::{DataCollector, DataExtractor, DataField, DataLocation, get_by_pointer},
        command::MinerCommand,
        device::{DeviceInfo, HashAlgorithm},
        fan::FanData,
        firmware::FirmwareImage,
        hashrate::{HashRate, HashRateUnit},
        message::{MessageSeverity, MinerMessage},
        miner::{MiningMode, TuningTarget},
        pool::{PoolData, PoolGroupData, PoolURL},
    },
    traits::{miner::*, model::MinerModel},
    util::unix_timestamp_secs,
};
use asic_rs_makes_proto::{
    hardware::{ProtoControlBoard, ProtoHardwareConfig},
    models::ProtoModel,
};
use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Power, Temperature, Voltage};
use reqwest::Method;
use serde_json::{Value, json};

use crate::firmware::ProtoFirmware;

mod web;

use web::ProtoWebAPI;

#[derive(Debug)]
pub struct ProtoV1 {
    ip: IpAddr,
    web: ProtoWebAPI,
    device_info: DeviceInfo,
    hardware_config: ProtoHardwareConfig,
}

impl ProtoV1 {
    pub fn new(ip: IpAddr, model: impl MinerModel) -> Self {
        let hardware_config = (&model as &dyn Any)
            .downcast_ref::<ProtoModel>()
            .map(|model| model.hardware().clone())
            .unwrap_or_default();
        let auth = Self::default_auth();

        Self {
            ip,
            web: ProtoWebAPI::new(ip, auth),
            device_info: DeviceInfo::new(model, ProtoFirmware::default(), HashAlgorithm::SHA256),
            hardware_config,
        }
    }

    pub fn new_with_hardware_config(
        ip: IpAddr,
        model_name: impl Into<String>,
        hardware_config: ProtoHardwareConfig,
    ) -> Self {
        Self::new(ip, ProtoModel::with_hardware(model_name, hardware_config))
    }

    fn response_success(value: &Value) -> bool {
        value
            .get("result")
            .and_then(Value::as_bool)
            .or_else(|| value.get("success").and_then(Value::as_bool))
            .unwrap_or_else(|| {
                value.get("message").is_some()
                    || value.as_object().is_some_and(serde_json::Map::is_empty)
            })
    }

    fn field_tag<'a>(
        data: &'a HashMap<DataField, Value>,
        field: DataField,
        tag: &str,
    ) -> Option<&'a Value> {
        data.get(&field).and_then(|value| value.get(tag))
    }

    fn config_value(data: &HashMap<ConfigField, Value>, field: ConfigField) -> Option<&Value> {
        data.get(&field)
    }

    fn position_from_slot(value: Option<u64>, fallback: usize) -> u8 {
        value
            .and_then(|slot| slot.checked_sub(1))
            .and_then(|position| u8::try_from(position).ok())
            .unwrap_or_else(|| u8::try_from(fallback).unwrap_or(u8::MAX))
    }

    fn hashrate(value: f64, unit: HashRateUnit) -> HashRate {
        HashRate {
            value,
            unit,
            algo: "SHA256".to_string(),
        }
        .as_unit(HashRateUnit::default())
    }

    fn metric_value(value: &Value) -> Option<f64> {
        value.get("value").and_then(Value::as_f64)
    }

    fn metric_hashrate(value: &Value) -> Option<HashRate> {
        let metric_value = Self::metric_value(value)?;
        let unit = value
            .get("unit")
            .and_then(Value::as_str)
            .and_then(|unit| HashRateUnit::from_str(unit).ok())
            .unwrap_or(HashRateUnit::TeraHash);
        Some(Self::hashrate(metric_value, unit))
    }

    fn board_mut(boards: &mut BTreeMap<u8, BoardData>, position: u8) -> &mut BoardData {
        boards
            .entry(position)
            .or_insert_with(|| BoardData::new(position, None))
    }

    fn parse_pool_configs(pools: &[Value]) -> Vec<PoolGroupConfig> {
        let configs = pools
            .iter()
            .filter_map(|pool| {
                let url = pool.get("url").and_then(Value::as_str)?;
                let username = pool
                    .get("username")
                    .or_else(|| pool.get("user"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let password = pool
                    .get("password")
                    .and_then(Value::as_str)
                    .unwrap_or("x")
                    .to_string();
                Some(PoolConfig {
                    url: PoolURL::from(url.to_string()),
                    username,
                    password,
                })
            })
            .collect::<Vec<_>>();

        if configs.is_empty() {
            vec![]
        } else {
            vec![PoolGroupConfig {
                name: String::new(),
                quota: 1,
                pools: configs,
            }]
        }
    }

    fn pools_payload(config: &[PoolGroupConfig]) -> Vec<Value> {
        let mut priority = 0u64;
        let mut payload = Vec::new();

        for group in config {
            for pool in &group.pools {
                payload.push(json!({
                        "name": group.name,
                        "url": pool.url.to_string(),
                        "username": pool.username,
                        "password": pool.password,
                        "priority": priority,
                }));
                priority += 1;
            }
        }

        payload
    }

    fn parse_mining_mode(mode: MiningMode) -> &'static str {
        match mode {
            MiningMode::Low | MiningMode::Normal => "Efficiency",
            MiningMode::High => "MaximumHashrate",
        }
    }
}

#[async_trait]
impl APIClient for ProtoV1 {
    async fn get_api_result(&self, command: &MinerCommand) -> anyhow::Result<Value> {
        match command {
            MinerCommand::WebAPI { .. } => self.web.get_api_result(command).await,
            _ => Err(anyhow::anyhow!(
                "Unsupported command type for Proto MDK API"
            )),
        }
    }
}

impl GetConfigsLocations for ProtoV1 {
    fn get_configs_locations(&self, config_field: ConfigField) -> Vec<ConfigLocation> {
        const WEB_POOLS: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/pools",
            parameters: None,
        };
        const WEB_TARGET: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/mining/target",
            parameters: None,
        };
        const WEB_COOLING: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/cooling",
            parameters: None,
        };

        match config_field {
            ConfigField::Pools => vec![(
                WEB_POOLS,
                ConfigExtractor {
                    func: get_config_by_pointer,
                    key: Some("/pools"),
                    tag: None,
                },
            )],
            ConfigField::Tuning => vec![(
                WEB_TARGET,
                ConfigExtractor {
                    func: get_config_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            ConfigField::Fan => vec![(
                WEB_COOLING,
                ConfigExtractor {
                    func: get_config_by_pointer,
                    key: Some("/cooling-status"),
                    tag: None,
                },
            )],
            ConfigField::Scaling => vec![],
        }
    }
}

impl CollectConfigs for ProtoV1 {
    fn get_config_collector(&self) -> ConfigCollector<'_> {
        ConfigCollector::new(self)
    }
}

impl GetDataLocations for ProtoV1 {
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        const WEB_SYSTEM: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/system",
            parameters: None,
        };
        const WEB_NETWORK: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/network",
            parameters: None,
        };
        const WEB_MINING: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/mining",
            parameters: None,
        };
        const WEB_HARDWARE: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/hardware",
            parameters: None,
        };
        const WEB_HASHBOARDS: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/hashboards",
            parameters: None,
        };
        const WEB_COOLING: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/cooling",
            parameters: None,
        };
        const WEB_ERRORS: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/errors",
            parameters: None,
        };
        const WEB_POOLS: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/pools",
            parameters: None,
        };
        const WEB_TELEMETRY: MinerCommand = MinerCommand::WebAPI {
            command: "/api/v1/telemetry",
            parameters: None,
        };

        match data_field {
            DataField::Mac => vec![(
                WEB_NETWORK,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/network-info/mac"),
                    tag: None,
                },
            )],
            DataField::SerialNumber => vec![(
                WEB_SYSTEM,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/system-info/cb_sn"),
                    tag: None,
                },
            )],
            DataField::Hostname => vec![(
                WEB_NETWORK,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/network-info/hostname"),
                    tag: None,
                },
            )],
            DataField::ApiVersion => vec![(
                WEB_HARDWARE,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/hardware-info/hashboards-info/0/api_version"),
                    tag: None,
                },
            )],
            DataField::FirmwareVersion => vec![(
                WEB_SYSTEM,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/system-info"),
                    tag: None,
                },
            )],
            DataField::ControlBoardVersion => vec![
                (
                    WEB_SYSTEM,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/system-info/board"),
                        tag: Some("system_board"),
                    },
                ),
                (
                    WEB_HARDWARE,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/hardware-info/cb-info/machine_name"),
                        tag: Some("machine_name"),
                    },
                ),
            ],
            DataField::Hashboards => vec![
                (
                    WEB_HARDWARE,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/hardware-info"),
                        tag: Some("hardware"),
                    },
                ),
                (
                    WEB_HASHBOARDS,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/hashboards-info"),
                        tag: Some("hashboards_info"),
                    },
                ),
                (
                    WEB_MINING,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/mining-status"),
                        tag: Some("mining"),
                    },
                ),
            ],
            DataField::Chips => vec![(
                WEB_TELEMETRY,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::Hashrate | DataField::ExpectedHashrate | DataField::Wattage => vec![(
                WEB_MINING,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/mining-status"),
                    tag: None,
                },
            )],
            DataField::TuningTarget | DataField::Uptime | DataField::IsMining => vec![(
                WEB_MINING,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/mining-status"),
                    tag: None,
                },
            )],
            DataField::Fans => vec![(
                WEB_COOLING,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/cooling-status"),
                    tag: None,
                },
            )],
            DataField::Messages => vec![(
                WEB_ERRORS,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                    tag: None,
                },
            )],
            DataField::Pools => vec![(
                WEB_POOLS,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/pools"),
                    tag: None,
                },
            )],
            _ => vec![],
        }
    }
}

impl GetIP for ProtoV1 {
    fn get_ip(&self) -> IpAddr {
        self.ip
    }
}

impl GetDeviceInfo for ProtoV1 {
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }
}

impl CollectData for ProtoV1 {
    fn get_collector(&self) -> DataCollector<'_> {
        DataCollector::new(self)
    }
}

impl GetMAC for ProtoV1 {
    fn parse_mac(&self, data: &HashMap<DataField, Value>) -> Option<MacAddr> {
        data.get(&DataField::Mac)
            .and_then(Value::as_str)
            .and_then(|mac| MacAddr::from_str(mac).ok())
    }
}

impl GetSerialNumber for ProtoV1 {
    fn parse_serial_number(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.get(&DataField::SerialNumber)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    }
}

impl GetHostname for ProtoV1 {
    fn parse_hostname(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.get(&DataField::Hostname)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    }
}

impl GetApiVersion for ProtoV1 {
    fn parse_api_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.get(&DataField::ApiVersion)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    }
}

impl GetFirmwareVersion for ProtoV1 {
    fn parse_firmware_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        let system = data.get(&DataField::FirmwareVersion)?;
        for pointer in [
            "/mining_driver_sw/version",
            "/web_server/version",
            "/web_dashboard/version",
            "/hashboard_firmware/version",
            "/pool_interface_sw/version",
            "/os/version",
        ] {
            if let Some(version) = system.pointer(pointer).and_then(Value::as_str) {
                return Some(version.to_string());
            }
        }
        None
    }
}

impl GetControlBoardVersion for ProtoV1 {
    fn parse_control_board_version(
        &self,
        data: &HashMap<DataField, Value>,
    ) -> Option<MinerControlBoard> {
        let control = data.get(&DataField::ControlBoardVersion)?;
        control
            .get("system_board")
            .and_then(Value::as_str)
            .or_else(|| control.get("machine_name").and_then(Value::as_str))
            .map(|name| ProtoControlBoard::parse(name).into())
    }
}

impl GetHashboards for ProtoV1 {
    fn parse_hashboards(&self, data: &HashMap<DataField, Value>) -> Vec<BoardData> {
        let mut boards = BTreeMap::new();
        for configured in &self.hardware_config.hashboards {
            let board = Self::board_mut(&mut boards, configured.position);
            board.expected_chips = configured.chips;
            board.serial_number = configured.serial_number.clone();
        }

        let hashboard_data = data.get(&DataField::Hashboards);

        let hardware = hashboard_data.and_then(|value| value.get("hardware"));
        let hashboard_infos = Self::field_tag(data, DataField::Hashboards, "hashboards_info")
            .and_then(Value::as_array)
            .or_else(|| {
                hardware
                    .and_then(|value| value.get("hashboards-info"))
                    .and_then(Value::as_array)
            });

        if let Some(hashboard_infos) = hashboard_infos {
            for (idx, info) in hashboard_infos.iter().enumerate() {
                let position = Self::position_from_slot(
                    info.get("slot")
                        .and_then(Value::as_u64)
                        .or_else(|| info.get("port").and_then(Value::as_u64)),
                    idx,
                );
                let board = Self::board_mut(&mut boards, position);
                board.serial_number = info
                    .get("hb_sn")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .or_else(|| board.serial_number.clone());
                board.expected_chips = info
                    .get("mining_asic_count")
                    .and_then(Value::as_u64)
                    .and_then(|chips| u16::try_from(chips).ok())
                    .or(board.expected_chips);
                board.active.get_or_insert(true);
            }
        }

        if let Some(telemetry_boards) = data
            .get(&DataField::Chips)
            .and_then(|value| value.get("hashboards"))
            .and_then(Value::as_array)
        {
            for (idx, telemetry_board) in telemetry_boards.iter().enumerate() {
                let position = telemetry_board
                    .get("index")
                    .and_then(Value::as_u64)
                    .and_then(|position| u8::try_from(position).ok())
                    .unwrap_or_else(|| u8::try_from(idx).unwrap_or(u8::MAX));
                let board = Self::board_mut(&mut boards, position);

                board.serial_number = telemetry_board
                    .get("serial_number")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .or_else(|| board.serial_number.clone());
                board.hashrate = telemetry_board
                    .get("hashrate")
                    .and_then(Self::metric_hashrate)
                    .or_else(|| board.hashrate.clone());
                board.board_temperature = telemetry_board
                    .get("temperature")
                    .and_then(|value| value.get("average").and_then(Value::as_f64))
                    .map(Temperature::from_celsius)
                    .or(board.board_temperature);
                board.intake_temperature = telemetry_board
                    .get("temperature")
                    .and_then(|value| value.get("inlet").and_then(Value::as_f64))
                    .map(Temperature::from_celsius)
                    .or(board.intake_temperature);
                board.outlet_temperature = telemetry_board
                    .get("temperature")
                    .and_then(|value| value.get("outlet").and_then(Value::as_f64))
                    .map(Temperature::from_celsius)
                    .or(board.outlet_temperature);
                board.voltage = telemetry_board
                    .get("voltage")
                    .and_then(Self::metric_value)
                    .map(Voltage::from_volts)
                    .or(board.voltage);
                board.active.get_or_insert(true);
                board.tuned.get_or_insert(true);

                if let Some(asics) = telemetry_board.get("asics") {
                    let hashrates = asics
                        .get("hashrate")
                        .and_then(|metric| metric.get("values"))
                        .and_then(Value::as_array);
                    let temperatures = asics
                        .get("temperature")
                        .and_then(|metric| metric.get("values"))
                        .and_then(Value::as_array);
                    let chip_count = hashrates
                        .map(Vec::len)
                        .unwrap_or(0)
                        .max(temperatures.map(Vec::len).unwrap_or(0));

                    if chip_count > 0 {
                        board.chips = (0..chip_count)
                            .filter_map(|position| {
                                let position_u16 = u16::try_from(position).ok()?;
                                Some(ChipData {
                                    position: position_u16,
                                    hashrate: hashrates
                                        .and_then(|values| values.get(position))
                                        .and_then(Value::as_f64)
                                        .map(|value| Self::hashrate(value, HashRateUnit::TeraHash)),
                                    temperature: temperatures
                                        .and_then(|values| values.get(position))
                                        .and_then(Value::as_f64)
                                        .map(Temperature::from_celsius),
                                    working: Some(true),
                                    tuned: Some(true),
                                    ..Default::default()
                                })
                            })
                            .collect();
                        board.working_chips = u16::try_from(board.chips.len()).ok();
                        board.expected_chips = board.expected_chips.or(board.working_chips);
                    }
                }
            }
        }

        let mining = Self::field_tag(data, DataField::Hashboards, "mining");
        let installed_boards = mining
            .and_then(|value| value.get("hashboards_installed"))
            .and_then(Value::as_u64)
            .and_then(|count| u8::try_from(count).ok())
            .or(self.device_info.hardware.boards);
        if boards.is_empty()
            && let Some(count) = installed_boards
        {
            for position in 0..count {
                let mut board =
                    BoardData::new(position, self.hardware_config.chips_for_position(position));
                board.active = Some(true);
                boards.insert(position, board);
            }
        }

        boards.into_values().collect()
    }
}

impl GetHashrate for ProtoV1 {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.get(&DataField::Hashrate)
            .and_then(|value| value.get("average_hashrate_ghs"))
            .and_then(Value::as_f64)
            .map(|value| Self::hashrate(value, HashRateUnit::GigaHash))
    }
}

impl GetExpectedHashrate for ProtoV1 {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.get(&DataField::ExpectedHashrate)
            .and_then(|value| value.get("ideal_hashrate_ghs"))
            .and_then(Value::as_f64)
            .map(|value| Self::hashrate(value, HashRateUnit::GigaHash))
    }
}

impl GetFans for ProtoV1 {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        data.get(&DataField::Fans)
            .and_then(|value| value.get("fans"))
            .and_then(Value::as_array)
            .map(|fans| {
                fans.iter()
                    .enumerate()
                    .filter_map(|(idx, fan)| {
                        let position = fan
                            .get("slot")
                            .and_then(Value::as_i64)
                            .and_then(|slot| slot.checked_sub(1))
                            .and_then(|position| i16::try_from(position).ok())
                            .unwrap_or_else(|| i16::try_from(idx).unwrap_or(i16::MAX));
                        Some(FanData {
                            position,
                            rpm: fan
                                .get("rpm")
                                .and_then(Value::as_f64)
                                .map(AngularVelocity::from_rpm),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl GetPsuFans for ProtoV1 {}

impl GetFluidTemperature for ProtoV1 {}

impl GetWattage for ProtoV1 {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.get(&DataField::Wattage)
            .and_then(|value| value.get("power_usage_watts"))
            .and_then(Value::as_f64)
            .map(Power::from_watts)
    }
}

impl GetTuningTarget for ProtoV1 {
    fn parse_tuning_target(&self, data: &HashMap<DataField, Value>) -> Option<TuningTarget> {
        data.get(&DataField::TuningTarget)
            .and_then(|value| value.get("power_target_watts"))
            .and_then(Value::as_f64)
            .map(TuningTarget::from_watts)
    }
}

impl GetLightFlashing for ProtoV1 {}

impl GetMessages for ProtoV1 {
    fn parse_messages(&self, data: &HashMap<DataField, Value>) -> Vec<MinerMessage> {
        let timestamp = unix_timestamp_secs() as u32;
        data.get(&DataField::Messages)
            .and_then(Value::as_array)
            .map(|messages| {
                messages
                    .iter()
                    .map(|message| {
                        let error_code = message
                            .get("error_code")
                            .and_then(Value::as_str)
                            .unwrap_or("ProtoMDK");
                        let severity = if error_code.to_ascii_lowercase().contains("warn") {
                            MessageSeverity::Warning
                        } else {
                            MessageSeverity::Error
                        };

                        MinerMessage {
                            timestamp: message
                                .get("timestamp")
                                .and_then(Value::as_u64)
                                .and_then(|timestamp| u32::try_from(timestamp).ok())
                                .unwrap_or(timestamp),
                            code: 0,
                            message: message
                                .get("message")
                                .and_then(Value::as_str)
                                .unwrap_or(error_code)
                                .to_string(),
                            severity,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl GetUptime for ProtoV1 {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.get(&DataField::Uptime)
            .and_then(|value| {
                value
                    .get("reboot_uptime_s")
                    .or_else(|| value.get("mining_uptime_s"))
            })
            .and_then(Value::as_u64)
            .map(Duration::from_secs)
    }
}

impl GetIsMining for ProtoV1 {
    fn parse_is_mining(&self, data: &HashMap<DataField, Value>) -> bool {
        data.get(&DataField::IsMining)
            .and_then(|value| value.get("status"))
            .and_then(Value::as_str)
            .is_some_and(|status| matches!(status, "Mining" | "DegradedMining" | "PoweringOn"))
            || self
                .parse_hashrate(data)
                .is_some_and(|hashrate| hashrate.value > 0.0)
    }
}

impl GetPools for ProtoV1 {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolGroupData> {
        let Some(pools) = data.get(&DataField::Pools).and_then(Value::as_array) else {
            return vec![];
        };

        let pools = pools
            .iter()
            .filter_map(|pool| {
                let url = pool
                    .get("url")
                    .and_then(Value::as_str)
                    .map(|url| PoolURL::from(url.to_string()));
                Some(PoolData {
                    position: pool
                        .get("id")
                        .or_else(|| pool.get("priority"))
                        .and_then(Value::as_u64)
                        .and_then(|position| u16::try_from(position).ok()),
                    url,
                    accepted_shares: pool.get("accepted").and_then(Value::as_u64),
                    rejected_shares: pool.get("rejected").and_then(Value::as_u64),
                    active: pool
                        .get("status")
                        .and_then(Value::as_str)
                        .map(|status| status == "Active"),
                    alive: pool
                        .get("status")
                        .and_then(Value::as_str)
                        .map(|status| status != "Dead"),
                    user: pool
                        .get("user")
                        .or_else(|| pool.get("username"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                })
            })
            .collect::<Vec<_>>();

        if pools.is_empty() {
            vec![]
        } else {
            vec![PoolGroupData {
                name: String::new(),
                quota: 1,
                pools,
            }]
        }
    }
}

#[async_trait]
impl SetFaultLight for ProtoV1 {
    async fn set_fault_light(&self, fault: bool) -> anyhow::Result<bool> {
        if !fault {
            return Ok(false);
        }
        self.web
            .send_command("/api/v1/system/locate", true, None, Method::POST)
            .await
            .map(|value| Self::response_success(&value))
    }

    fn supports_set_fault_light(&self) -> bool {
        true
    }
}

#[async_trait]
impl SetPowerLimit for ProtoV1 {
    async fn set_power_limit(&self, limit: Power) -> anyhow::Result<bool> {
        self.web
            .send_command(
                "/api/v1/mining/target",
                true,
                Some(json!({ "power_target_watts": limit.as_watts().round() as u64 })),
                Method::PUT,
            )
            .await
            .map(|value| Self::response_success(&value))
    }

    fn supports_set_power_limit(&self) -> bool {
        true
    }
}

#[async_trait]
impl Restart for ProtoV1 {
    async fn restart(&self) -> anyhow::Result<bool> {
        self.web
            .send_command("/api/v1/system/reboot", true, None, Method::POST)
            .await
            .map(|value| Self::response_success(&value))
    }

    fn supports_restart(&self) -> bool {
        true
    }
}

#[async_trait]
impl Pause for ProtoV1 {
    async fn pause(&self, _at_time: Option<Duration>) -> anyhow::Result<bool> {
        self.web
            .send_command("/api/v1/mining/stop", true, None, Method::POST)
            .await
            .map(|value| Self::response_success(&value))
    }

    fn supports_pause(&self) -> bool {
        true
    }
}

#[async_trait]
impl Resume for ProtoV1 {
    async fn resume(&self, _at_time: Option<Duration>) -> anyhow::Result<bool> {
        self.web
            .send_command("/api/v1/mining/start", true, None, Method::POST)
            .await
            .map(|value| Self::response_success(&value))
    }

    fn supports_resume(&self) -> bool {
        true
    }
}

#[async_trait]
impl ChangePassword for ProtoV1 {
    async fn change_password(&mut self, password: &str) -> anyhow::Result<bool> {
        let response = self
            .web
            .send_command(
                "/api/v1/auth/change-password",
                true,
                Some(json!({
                    "current_password": self.web.password(),
                    "new_password": password,
                })),
                Method::PUT,
            )
            .await?;

        let success = Self::response_success(&response);
        if success {
            self.set_auth(MinerAuth::new("", password));
        }
        Ok(success)
    }

    fn supports_change_password(&self) -> bool {
        true
    }
}

impl FactoryReset for ProtoV1 {
    fn supports_factory_reset(&self) -> bool {
        false
    }
}

#[async_trait]
impl ReadLogs for ProtoV1 {
    async fn read_logs(&self) -> anyhow::Result<String> {
        self.web.read_logs().await
    }

    fn supports_read_logs(&self) -> bool {
        true
    }
}

#[async_trait]
impl UpgradeFirmware for ProtoV1 {
    async fn upgrade_firmware(&self, image: FirmwareImage) -> anyhow::Result<bool> {
        self.web.upgrade_firmware(image).await
    }

    fn supports_upgrade_firmware(&self) -> bool {
        true
    }
}

#[async_trait]
impl SupportsPoolsConfig for ProtoV1 {
    async fn set_pools_config(&self, config: Vec<PoolGroupConfig>) -> anyhow::Result<bool> {
        let payload = Self::pools_payload(&config);
        self.web
            .send_command(
                "/api/v1/pools",
                true,
                Some(Value::Array(payload)),
                Method::POST,
            )
            .await
            .map(|value| Self::response_success(&value))
    }

    fn parse_pools_config(
        &self,
        data: &HashMap<ConfigField, Value>,
    ) -> anyhow::Result<Vec<PoolGroupConfig>> {
        let Some(pools) = Self::config_value(data, ConfigField::Pools).and_then(Value::as_array)
        else {
            return Ok(vec![]);
        };
        Ok(Self::parse_pool_configs(pools))
    }

    fn supports_pools_config(&self) -> bool {
        true
    }
}

#[async_trait]
impl SupportsScalingConfig for ProtoV1 {
    async fn set_scaling_config(&self, _config: ScalingConfig) -> anyhow::Result<bool> {
        anyhow::bail!("Scaling config is not supported by the Proto MDK API")
    }

    fn supports_scaling_config(&self) -> bool {
        false
    }
}

#[async_trait]
impl SupportsTuningConfig for ProtoV1 {
    fn parse_tuning_config(
        &self,
        data: &HashMap<ConfigField, Value>,
    ) -> anyhow::Result<TuningConfig> {
        let watts = Self::config_value(data, ConfigField::Tuning)
            .and_then(|value| value.get("power_target_watts"))
            .and_then(Value::as_f64)
            .context("Proto MDK mining target did not include power_target_watts")?;
        Ok(TuningConfig::new(TuningTarget::from_watts(watts)))
    }

    async fn set_tuning_config(
        &self,
        config: TuningConfig,
        _scaling_config: Option<ScalingConfig>,
    ) -> anyhow::Result<bool> {
        if let Some(algorithm) = config.algorithm() {
            self.web
                .send_command(
                    "/api/v1/mining/tuning",
                    true,
                    Some(json!({ "algorithm": algorithm })),
                    Method::PUT,
                )
                .await?;
        }

        match config.target {
            TuningTarget::Power(power) => self.set_power_limit(power).await,
            TuningTarget::HashRate(_) => {
                anyhow::bail!("Hashrate tuning target is not supported by the Proto MDK API")
            }
            TuningTarget::MiningMode(mode) => self
                .web
                .send_command(
                    "/api/v1/mining/target",
                    true,
                    Some(json!({ "performance_mode": Self::parse_mining_mode(mode) })),
                    Method::PUT,
                )
                .await
                .map(|value| Self::response_success(&value)),
        }
    }

    fn supports_tuning_config(&self) -> bool {
        true
    }
}

#[async_trait]
impl SupportsFanConfig for ProtoV1 {
    fn parse_fan_config(&self, data: &HashMap<ConfigField, Value>) -> anyhow::Result<FanConfig> {
        let cooling = Self::config_value(data, ConfigField::Fan)
            .context("Proto MDK cooling config missing")?;
        match cooling
            .get("fan_mode")
            .or_else(|| cooling.get("mode"))
            .and_then(Value::as_str)
            .unwrap_or("Unknown")
        {
            "Auto" => {
                let target_temp = cooling
                    .get("target_temperature_c")
                    .and_then(Value::as_f64)
                    .unwrap_or(50.0);
                Ok(FanConfig::auto(target_temp, None))
            }
            "Manual" => {
                let speed = cooling
                    .get("speed_percentage")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                Ok(FanConfig::manual(speed))
            }
            "Off" => Ok(FanConfig::manual(0)),
            other => anyhow::bail!("Unsupported Proto MDK fan mode {other}"),
        }
    }

    async fn set_fan_config(&self, config: FanConfig) -> anyhow::Result<bool> {
        let payload = match config {
            FanConfig::Auto { target_temp, .. } => {
                json!({ "mode": "Auto", "target_temperature_c": target_temp })
            }
            FanConfig::Manual { fan_speed } => {
                json!({ "mode": "Manual", "speed_percentage": fan_speed.min(100) })
            }
        };

        self.web
            .send_command("/api/v1/cooling", true, Some(payload), Method::PUT)
            .await
            .map(|value| Self::response_success(&value))
    }

    fn supports_fan_config(&self) -> bool {
        true
    }
}

impl HasAuth for ProtoV1 {
    fn set_auth(&mut self, auth: MinerAuth) {
        self.web.set_auth(auth);
    }
}

impl HasDefaultAuth for ProtoV1 {}

#[cfg(test)]
mod tests {
    use asic_rs_core::test::api::MockAPIClient;
    use asic_rs_makes_proto::hardware::{ProtoHashboardConfig, ProtoHashboardKind};

    use super::*;

    #[tokio::test]
    async fn parses_heterogeneous_hashboards_from_mdk_shapes() {
        let hardware = ProtoHardwareConfig::new(
            vec![
                ProtoHashboardConfig::new(0, ProtoHashboardKind::B3a, Some(100)),
                ProtoHashboardConfig::new(1, ProtoHashboardKind::B4, Some(120)),
            ],
            Some(4),
        );
        let miner =
            ProtoV1::new_with_hardware_config(IpAddr::from([127, 0, 0, 1]), "Rig", hardware);

        let mut responses = HashMap::new();
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/system",
                parameters: None,
            },
            json!({
                "system-info": {
                    "product_name": "Proto Rig",
                    "manufacturer": "Proto",
                    "model": "Rig",
                    "cb_sn": "CB123",
                    "board": "C3",
                    "mining_driver_sw": { "version": "1.2.3" }
                }
            }),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/network",
                parameters: None,
            },
            json!({
                "network-info": {
                    "mac": "82:11:D2:94:0D:6D",
                    "hostname": "proto-miner-1"
                }
            }),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/mining",
                parameters: None,
            },
            json!({
                "mining-status": {
                    "status": "Mining",
                    "average_hashrate_ghs": 95000.0,
                    "ideal_hashrate_ghs": 100000.0,
                    "power_usage_watts": 3100.0,
                    "power_target_watts": 3200.0,
                    "reboot_uptime_s": 600,
                    "hashboards_installed": 2
                }
            }),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/hardware",
                parameters: None,
            },
            json!({
                "hardware-info": {
                    "hashboards-info": [
                        {
                            "hb_sn": "HB-A",
                            "board": "B3a",
                            "mining_asic": "BZM",
                            "mining_asic_count": 100,
                            "slot": 1,
                            "api_version": "1.0"
                        },
                        {
                            "hb_sn": "HB-B",
                            "board": "B4",
                            "mining_asic": "MC1",
                            "mining_asic_count": 120,
                            "slot": 2,
                            "api_version": "1.0"
                        }
                    ],
                    "fans-info": [{ "slot": 1 }, { "slot": 2 }, { "slot": 3 }, { "slot": 4 }],
                    "cb-info": { "machine_name": "C3", "serial_number": "CB123" }
                }
            }),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/hashboards",
                parameters: None,
            },
            json!({
                "hashboards-info": [
                    { "hb_sn": "HB-A", "mining_asic_count": 100, "slot": 1 },
                    { "hb_sn": "HB-B", "mining_asic_count": 120, "slot": 2 }
                ]
            }),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/telemetry",
                parameters: None,
            },
            json!({
                "timestamp": "2024-01-15T14:30:00Z",
                "hashboards": [
                    {
                        "index": 0,
                        "serial_number": "HB-A",
                        "hashrate": { "value": 45.0, "unit": "TH/s" },
                        "temperature": { "unit": "°C", "inlet": 40.0, "outlet": 70.0, "average": 55.0 },
                        "voltage": { "value": 12.1, "unit": "V" },
                        "asics": {
                            "hashrate": { "unit": "TH/s", "values": [0.45, 0.45] },
                            "temperature": { "unit": "°C", "values": [65.0, 66.0] }
                        }
                    },
                    {
                        "index": 1,
                        "serial_number": "HB-B",
                        "hashrate": { "value": 50.0, "unit": "TH/s" },
                        "temperature": { "unit": "°C", "inlet": 41.0, "outlet": 72.0, "average": 56.5 },
                        "voltage": { "value": 12.2, "unit": "V" }
                    }
                ]
            }),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/cooling",
                parameters: None,
            },
            json!({
                "cooling-status": {
                    "fan_mode": "Auto",
                    "target_temperature_c": 55.0,
                    "fans": [
                        { "slot": 1, "rpm": 1200 },
                        { "slot": 2, "rpm": 1210 }
                    ]
                }
            }),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/errors",
                parameters: None,
            },
            json!([]),
        );
        responses.insert(
            MinerCommand::WebAPI {
                command: "/api/v1/pools",
                parameters: None,
            },
            json!({ "pools": [] }),
        );

        let mock_api = MockAPIClient::new(responses);

        let mut collector = DataCollector::new_with_client(&miner, &mock_api);
        let data = collector.collect(&[DataField::Hashboards]).await;
        assert!(!data.contains_key(&DataField::Chips));
        let hashboards_without_chips = miner.parse_hashboards(&data);
        assert_eq!(hashboards_without_chips.len(), 2);
        assert!(hashboards_without_chips[0].chips.is_empty());
        assert_eq!(hashboards_without_chips[0].expected_chips, Some(100));
        assert_eq!(hashboards_without_chips[1].expected_chips, Some(120));

        let mut collector = DataCollector::new_with_client(&miner, &mock_api);
        let data = collector
            .collect(&[DataField::Hashboards, DataField::Chips])
            .await;
        assert!(data.contains_key(&DataField::Chips));
        let hashboards_with_chips = miner.parse_hashboards(&data);
        assert_eq!(hashboards_with_chips[0].chips.len(), 2);
        assert_eq!(
            hashboards_without_chips[0].expected_chips,
            hashboards_with_chips[0].expected_chips
        );

        let mut collector = DataCollector::new_with_client(&miner, &mock_api);
        let data = collector.collect_all().await;
        let miner_data = miner.parse_data(data);

        assert_eq!(miner_data.expected_hashboards, Some(2));
        assert_eq!(miner_data.expected_chips, None);
        assert_eq!(miner_data.hashboards.len(), 2);
        assert_eq!(miner_data.hashboards[0].expected_chips, Some(100));
        assert_eq!(miner_data.hashboards[1].expected_chips, Some(120));
        assert_eq!(
            miner_data.hashboards[0].serial_number.as_deref(),
            Some("HB-A")
        );
        assert_eq!(miner_data.hashboards[0].chips.len(), 2);
        assert!(miner_data.is_mining);
    }
}
