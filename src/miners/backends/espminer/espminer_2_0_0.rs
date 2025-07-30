use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};

use crate::data::board::{BoardData, ChipData};
use crate::data::device::MinerMake;
use crate::data::device::{DeviceInfo, HashAlgorithm, MinerFirmware, MinerModel};
use crate::data::fan::FanData;
use crate::data::hashrate::{HashRate, HashRateUnit};
use crate::data::message::{MessageSeverity, MinerMessage};
use crate::data::miner::MinerData;
use crate::data::pool::{PoolData, PoolScheme, PoolURL};
use crate::miners::api::web::espminer::ESPMinerWebAPI;
use crate::miners::backends::traits::GetMinerData;
use crate::miners::commands::MinerCommand;
use crate::miners::data::{
    DataCollector, DataExtensions, DataExtractor, DataField, DataLocation, get_by_key,
    get_by_pointer,
};

#[derive(Debug)]
pub struct ESPMiner200 {
    ip: IpAddr,
    web: ESPMinerWebAPI,
    device_info: DeviceInfo,
}

impl ESPMiner200 {
    pub fn new(ip: IpAddr, model: MinerModel, firmware: MinerFirmware) -> Self {
        ESPMiner200 {
            ip,
            web: ESPMinerWebAPI::new(ip, 80),
            device_info: DeviceInfo::new(MinerMake::BitAxe, model, firmware, HashAlgorithm::SHA256),
        }
    }
}

#[async_trait]
impl GetMinerData for ESPMiner200 {
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
        let is_mining = hashrate.as_ref().is_some_and(|hr| hr.value > 0.0);

        // Get hardware specifications based on the miner model
        let miner_hardware = self.device_info.hardware;

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
            expected_hashrate: None,

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

    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        let system_info_command: MinerCommand = MinerCommand::WebAPI {
            command: "system/info",
            parameters: None,
        };

        match data_field {
            DataField::Mac => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("macAddr"),
                },
            )],
            DataField::Hostname => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("hostname"),
                },
            )],
            DataField::FirmwareVersion => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("version"),
                },
            )],
            DataField::ApiVersion => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("version"),
                },
            )],
            DataField::ControlBoardVersion => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("boardVersion"),
                },
            )],
            DataField::Hashboards => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                },
            )],
            DataField::Hashrate => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("hashRate"),
                },
            )],
            DataField::TotalChips => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("asicCount"),
                },
            )],
            DataField::Fans => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("fanrpm"),
                },
            )],
            DataField::AverageTemperature => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("temp"),
                },
            )],
            DataField::Wattage => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("power"),
                },
            )],
            DataField::Uptime => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_key,
                    key: Some("uptimeSeconds"),
                },
            )],
            DataField::Pools => vec![(
                system_info_command,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some(""),
                },
            )],
            _ => vec![],
        }
    }
}
