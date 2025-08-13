use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};
use serde_json::Value;

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
use web::VnishWebAPI;

pub mod web;

#[derive(Debug)]
pub struct Vnish {
    ip: IpAddr,
    web: VnishWebAPI,
    device_info: DeviceInfo,
}

impl Vnish {
    pub fn new(ip: IpAddr, make: MinerMake, model: MinerModel) -> Self {
        Vnish {
            ip,
            web: VnishWebAPI::new(ip, 80),
            device_info: DeviceInfo::new(make, model, MinerFirmware::VNish, HashAlgorithm::SHA256),
        }
    }
}

impl GetDataLocations for Vnish {
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation> {
        fn cmd(endpoint: &'static str) -> MinerCommand {
            MinerCommand::WebAPI {
                command: endpoint,
                parameters: None,
            }
        }

        let info_cmd = cmd("info");
        let status_cmd = cmd("status");
        let summary_cmd = cmd("summary");
        let chains_cmd = cmd("chains");
        let factory_info_cmd = cmd("chains/factory-info");

        match data_field {
            DataField::Mac => vec![(
                info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/system/network_status/mac"),
                    tag: None,
                },
            )],
            DataField::SerialNumber => vec![
                (
                    factory_info_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/psu_serial"),
                        tag: None,
                    },
                ),
                (
                    info_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/serial"),
                        tag: None,
                    },
                ),
            ],
            DataField::Hostname => vec![(
                info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/system/network_status/hostname"),
                    tag: None,
                },
            )],
            DataField::ApiVersion => vec![(
                info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/fw_version"),
                    tag: None,
                },
            )],
            DataField::FirmwareVersion => vec![(
                info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/fw_version"),
                    tag: None,
                },
            )],
            DataField::ControlBoardVersion => vec![(
                info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/platform"),
                    tag: None,
                },
            )],
            DataField::Uptime => vec![(
                info_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/system/uptime"),
                    tag: None,
                },
            )],
            DataField::Hashrate => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/hr_realtime"),
                    tag: None,
                },
            )],
            DataField::ExpectedHashrate => vec![
                (
                    factory_info_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/hr_stock"),
                        tag: None,
                    },
                ),
                (
                    summary_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/miner/hr_stock"),
                        tag: None,
                    },
                ),
            ],
            DataField::Wattage => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/power_consumption"),
                    tag: None,
                },
            )],
            DataField::Fans => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/cooling/fans"),
                    tag: None,
                },
            )],
            DataField::Hashboards => vec![
                (
                    summary_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some("/miner/chains"),
                        tag: None,
                    },
                ),
                (
                    chains_cmd,
                    DataExtractor {
                        func: get_by_pointer,
                        key: Some(""),
                        tag: None,
                    },
                ),
            ],
            DataField::Pools => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/pools"),
                    tag: None,
                },
            )],
            DataField::IsMining => vec![(
                status_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner_state"),
                    tag: None,
                },
            )],
            DataField::Efficiency => vec![(
                summary_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/miner/power_efficiency"),
                    tag: None,
                },
            )],
            DataField::LightFlashing => vec![(
                status_cmd,
                DataExtractor {
                    func: get_by_pointer,
                    key: Some("/find_miner"),
                    tag: None,
                },
            )],
            _ => vec![],
        }
    }
}

impl GetIP for Vnish {
    fn get_ip(&self) -> IpAddr {
        self.ip
    }
}

impl GetDeviceInfo for Vnish {
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info
    }
}

impl CollectData for Vnish {
    fn get_collector(&self) -> DataCollector<'_> {
        DataCollector::new(self, &self.web)
    }
}

impl GetMAC for Vnish {
    fn parse_mac(&self, data: &HashMap<DataField, Value>) -> Option<MacAddr> {
        data.extract::<String>(DataField::Mac)
            .and_then(|s| MacAddr::from_str(&s).ok())
    }
}

impl GetSerialNumber for Vnish {
    fn parse_serial_number(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::SerialNumber)
    }
}

impl GetHostname for Vnish {
    fn parse_hostname(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::Hostname)
    }
}

impl GetApiVersion for Vnish {
    fn parse_api_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::ApiVersion)
    }
}

impl GetFirmwareVersion for Vnish {
    fn parse_firmware_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::FirmwareVersion)
    }
}

impl GetControlBoardVersion for Vnish {
    fn parse_control_board_version(&self, data: &HashMap<DataField, Value>) -> Option<String> {
        data.extract::<String>(DataField::ControlBoardVersion)
    }
}

impl GetHashboards for Vnish {
    fn parse_hashboards(&self, data: &HashMap<DataField, Value>) -> Vec<BoardData> {
        let mut hashboards: Vec<BoardData> = Vec::new();

        let chains_data = data.get(&DataField::Hashboards).and_then(|v| v.as_array());

        if let Some(chains_array) = chains_data {
            for (idx, chain) in chains_array.iter().enumerate() {
                let hashrate = Self::extract_hashrate(chain, &["/hashrate_rt", "/hr_realtime"]);
                let expected_hashrate =
                    Self::extract_hashrate(chain, &["/hashrate_ideal", "/hr_nominal"]);

                let frequency = Self::extract_frequency(chain);
                let voltage = Self::extract_voltage(chain);
                let (board_temperature, chip_temperature) = Self::extract_temperatures(chain);

                let working_chips = Self::extract_working_chips(chain);
                let active = Self::extract_chain_active_status(chain, &hashrate);
                let serial_number = Self::extract_chain_serial(chain, data);
                let tuned = Self::extract_tuned_status(chain, data);
                let chips = Self::extract_chips(chain);

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
                    chips,
                    voltage,
                    frequency,
                    tuned,
                    active,
                });
            }
        }

        hashboards
    }
}

impl GetHashrate for Vnish {
    fn parse_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::Hashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::GigaHash,
            algo: String::from("SHA256"),
        })
    }
}

impl GetExpectedHashrate for Vnish {
    fn parse_expected_hashrate(&self, data: &HashMap<DataField, Value>) -> Option<HashRate> {
        data.extract_map::<f64, _>(DataField::ExpectedHashrate, |f| HashRate {
            value: f,
            unit: HashRateUnit::GigaHash,
            algo: String::from("SHA256"),
        })
    }
}

impl GetFans for Vnish {
    fn parse_fans(&self, data: &HashMap<DataField, Value>) -> Vec<FanData> {
        let mut fans: Vec<FanData> = Vec::new();

        if let Some(fans_data) = data.get(&DataField::Fans)
            && let Some(fans_array) = fans_data.as_array()
        {
            for (idx, fan) in fans_array.iter().enumerate() {
                if let Some(rpm) = fan.pointer("/rpm").and_then(|v| v.as_i64()) {
                    fans.push(FanData {
                        position: idx as i16,
                        rpm: Some(AngularVelocity::from_rpm(rpm as f64)),
                    });
                }
            }
        }

        fans
    }
}

impl GetPsuFans for Vnish {}

impl GetFluidTemperature for Vnish {}

impl GetWattage for Vnish {
    fn parse_wattage(&self, data: &HashMap<DataField, Value>) -> Option<Power> {
        data.extract_map::<i64, _>(DataField::Wattage, |w| Power::from_watts(w as f64))
    }
}

impl GetWattageLimit for Vnish {}

impl GetLightFlashing for Vnish {
    fn parse_light_flashing(&self, data: &HashMap<DataField, Value>) -> Option<bool> {
        data.extract::<bool>(DataField::LightFlashing)
    }
}

impl GetMessages for Vnish {}

impl GetUptime for Vnish {
    fn parse_uptime(&self, data: &HashMap<DataField, Value>) -> Option<Duration> {
        data.extract::<String>(DataField::Uptime)
            .and_then(|uptime_str| {
                // Parse uptime strings like "10 days, 18:00"
                let trimmed = uptime_str.trim();

                // Try to parse format like "X days, HH:MM" or "X days"
                if trimmed.contains("days") {
                    let mut total_seconds = 0u64;

                    // Extract days
                    if let Some(days_part) = trimmed.split("days").next()
                        && let Ok(days) = days_part.trim().parse::<u64>()
                    {
                        total_seconds += days * 24 * 60 * 60;
                    }

                    // Extract hours and minutes if present (after comma)
                    if let Some(time_part) = trimmed.split(',').nth(1) {
                        let time_part = time_part.trim();
                        if let Some((hours_str, minutes_str)) = time_part.split_once(':')
                            && let (Ok(hours), Ok(minutes)) = (
                                hours_str.trim().parse::<u64>(),
                                minutes_str.trim().parse::<u64>(),
                            )
                        {
                            total_seconds += hours * 60 * 60 + minutes * 60;
                        }
                    }

                    return Some(Duration::from_secs(total_seconds));
                }

                None
            })
    }
}

impl GetIsMining for Vnish {
    fn parse_is_mining(&self, data: &HashMap<DataField, Value>) -> bool {
        data.extract::<String>(DataField::IsMining)
            .map(|state| state == "mining")
            .unwrap_or(false)
    }
}

impl GetPools for Vnish {
    fn parse_pools(&self, data: &HashMap<DataField, Value>) -> Vec<PoolData> {
        let mut pools: Vec<PoolData> = Vec::new();

        if let Some(pools_data) = data.get(&DataField::Pools)
            && let Some(pools_array) = pools_data.as_array()
        {
            for (idx, pool) in pools_array.iter().enumerate() {
                let url = pool
                    .pointer("/url")
                    .and_then(|v| v.as_str())
                    .map(Self::parse_pool_url);

                let user = pool
                    .pointer("/user")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let accepted_shares = pool.pointer("/accepted").and_then(|v| v.as_u64());
                let rejected_shares = pool.pointer("/rejected").and_then(|v| v.as_u64());
                let pool_status = pool.pointer("/status").and_then(|v| v.as_str());
                let (active, alive) = Self::parse_pool_status(pool_status);

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

// Helper methods for data extraction
impl Vnish {
    fn extract_hashrate(chain: &Value, paths: &[&str]) -> Option<HashRate> {
        paths
            .iter()
            .find_map(|&path| chain.pointer(path).and_then(|v| v.as_f64()))
            .map(|f| HashRate {
                value: f,
                unit: HashRateUnit::GigaHash,
                algo: String::from("SHA256"),
            })
    }

    fn extract_frequency(chain: &Value) -> Option<Frequency> {
        chain
            .pointer("/frequency")
            .or_else(|| chain.pointer("/freq"))
            .and_then(|v| v.as_f64())
            .map(Frequency::from_megahertz)
    }

    fn extract_voltage(chain: &Value) -> Option<Voltage> {
        chain
            .pointer("/voltage")
            .and_then(|v| v.as_i64())
            .map(|v| Voltage::from_millivolts(v as f64))
    }

    fn extract_temperatures(chain: &Value) -> (Option<Temperature>, Option<Temperature>) {
        let board_temp = chain
            .pointer("/pcb_temp/max")
            .and_then(|v| v.as_i64())
            .map(|t| Temperature::from_celsius(t as f64));

        let chip_temp = chain
            .pointer("/chip_temp/max")
            .and_then(|v| v.as_i64())
            .map(|t| Temperature::from_celsius(t as f64));

        (board_temp, chip_temp)
    }

    fn extract_working_chips(chain: &Value) -> Option<u16> {
        chain
            .pointer("/chip_statuses")
            .map(|statuses| {
                let red = statuses
                    .pointer("/red")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let orange = statuses
                    .pointer("/orange")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                (red + orange) as u16
            })
            .or_else(|| {
                chain
                    .pointer("/chips")
                    .and_then(|v| v.as_array())
                    .map(|chips| chips.len() as u16)
            })
    }

    fn extract_chain_active_status(chain: &Value, hashrate: &Option<HashRate>) -> Option<bool> {
        chain
            .pointer("/status/state")
            .and_then(|v| v.as_str())
            .map(|s| s == "mining")
            .or_else(|| hashrate.as_ref().map(|h| h.value > 0.0))
    }

    fn parse_pool_url(url_str: &str) -> PoolURL {
        // Convert host:port format to full UR
        let full_url = if url_str.starts_with("stratum") {
            url_str.to_string()
        } else {
            format!("stratum+tcp://{}", url_str)
        };

        PoolURL::from(full_url)
    }

    fn parse_pool_status(status: Option<&str>) -> (Option<bool>, Option<bool>) {
        match status {
            Some("active" | "working") => (Some(true), Some(true)),
            Some("offline" | "disabled") => (Some(false), Some(false)),
            Some("rejecting") => (Some(false), Some(true)),
            _ => (None, None),
        }
    }

    fn extract_chain_serial(chain: &Value, data: &HashMap<DataField, Value>) -> Option<String> {
        // Try to get serial from chain-specific data first (factory-info)
        chain
            .pointer("/serial")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| {
                // Fallback to miner-wide serial number
                data.extract::<String>(DataField::SerialNumber)
            })
    }

    fn extract_tuned_status(_chain: &Value, data: &HashMap<DataField, Value>) -> Option<bool> {
        // Check miner state to determine tuning status
        if let Some(miner_state) = data.extract::<String>(DataField::IsMining) {
            match miner_state.as_str() {
                "auto-tuning" => Some(false), // Currently tuning, not yet tuned
                "mining" => Some(true),       // Tuned and mining
                _ => None,
            }
        } else {
            None
        }
    }

    fn extract_chips(chain: &Value) -> Vec<ChipData> {
        let mut chips: Vec<ChipData> = Vec::new();

        if let Some(chips_array) = chain.pointer("/chips").and_then(|v| v.as_array()) {
            for (idx, chip) in chips_array.iter().enumerate() {
                let hashrate = chip
                    .pointer("/hr")
                    .and_then(|v| v.as_f64())
                    .map(|f| HashRate {
                        value: f,
                        unit: HashRateUnit::GigaHash,
                        algo: String::from("SHA256"),
                    });

                let temperature = chip
                    .pointer("/temp")
                    .and_then(|v| v.as_f64())
                    .map(Temperature::from_celsius);

                let voltage = chip
                    .pointer("/volt")
                    .and_then(|v| v.as_i64())
                    .map(|v| Voltage::from_millivolts(v as f64));

                let frequency = chip
                    .pointer("/freq")
                    .and_then(|v| v.as_i64())
                    .map(|f| Frequency::from_megahertz(f as f64));

                let working = hashrate.as_ref().map(|hr| hr.value > 0.0);

                chips.push(ChipData {
                    position: chip
                        .pointer("/id")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(idx as u64) as u16,
                    hashrate,
                    temperature,
                    voltage,
                    frequency,
                    tuned: None,
                    working,
                });
            }
        }

        chips
    }
}
