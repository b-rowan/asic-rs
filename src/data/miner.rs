use std::{net::IpAddr, time::Duration};

use macaddr::MacAddr;
use measurements::{Power, Temperature};

use super::{
    board::BoardData, device::DeviceInfo, fan::FanData, hashrate::HashRate, message::MinerMessage,
    pool::PoolData,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MinerData {
    pub schema_version: String,
    pub timestamp: u64,
    pub ip: IpAddr,
    pub mac: Option<MacAddr>,
    pub device_info: DeviceInfo,
    pub serial_number: Option<String>,
    pub hostname: Option<String>,
    pub api_version: Option<String>,
    pub firmware_version: Option<String>,
    pub control_board_version: Option<String>,
    pub expected_hashboards: Option<u8>,
    pub hashboards: Vec<BoardData>,
    pub hashrate: Option<HashRate>,
    pub expected_chips: Option<u16>,
    pub total_chips: Option<u16>,
    pub expected_fans: Option<u8>,
    pub fans: Vec<FanData>,
    pub psu_fans: Vec<FanData>,
    pub average_temperature: Option<Temperature>,
    pub fluid_temperature: Option<Temperature>,
    pub wattage: Option<Power>,
    pub wattage_limit: Option<Power>,
    pub efficiency: Option<f64>,
    pub light_flashing: Option<bool>,
    pub messages: Vec<MinerMessage>,
    pub uptime: Option<Duration>,
    pub is_mining: bool,
    pub pools: Vec<PoolData>,
}
