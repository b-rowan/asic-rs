use crate::data::deserialize::deserialize_macaddr;
use crate::data::serialize::serialize_macaddr;
use crate::data::serialize::serialize_power;
use crate::data::serialize::serialize_temperature;
use std::{net::IpAddr, time::Duration};

pub use super::{
    board::BoardData, device::DeviceInfo, fan::FanData, hashrate::HashRate, message::MinerMessage,
    pool::PoolData,
};
use macaddr::MacAddr;
use measurements::{Power, Temperature};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass(module = "asic_rs")]
pub struct MinerData {
    /// The schema version of this MinerData object, for use in external APIs
    #[pyo3(get)]
    pub schema_version: String,
    /// The time this data was gathered and constructed
    #[pyo3(get)]
    pub timestamp: u64,
    /// The IP address of the miner this data is for
    #[pyo3(get)]
    pub ip: IpAddr,
    /// The MAC address of the miner this data is for
    #[serde(
        serialize_with = "serialize_macaddr",
        deserialize_with = "deserialize_macaddr"
    )]
    pub mac: Option<MacAddr>,
    /// Hardware information about this miner
    #[pyo3(get)]
    pub device_info: DeviceInfo,
    /// The serial number of the miner, also known as the control board serial
    #[pyo3(get)]
    pub serial_number: Option<String>,
    /// The network hostname of the miner
    #[pyo3(get)]
    pub hostname: Option<String>,
    /// The API version of the miner
    #[pyo3(get)]
    pub api_version: Option<String>,
    /// The firmware version of the miner
    #[pyo3(get)]
    pub firmware_version: Option<String>,
    /// The type of control board on the miner
    #[pyo3(get)]
    pub control_board_version: Option<String>,
    /// The expected number of boards in the miner.
    #[pyo3(get)]
    pub expected_hashboards: Option<u8>,
    /// Per-hashboard data for this miner
    #[pyo3(get)]
    pub hashboards: Vec<BoardData>,
    /// The current hashrate of the miner
    #[pyo3(get)]
    pub hashrate: Option<HashRate>,
    /// The expected hashrate of the miner
    #[pyo3(get)]
    pub expected_hashrate: Option<HashRate>,
    /// The total expected number of chips across all boards on this miner
    #[pyo3(get)]
    pub expected_chips: Option<u16>,
    /// The total number of working chips across all boards on this miner
    #[pyo3(get)]
    pub total_chips: Option<u16>,
    /// The expected number of fans on the miner
    #[pyo3(get)]
    pub expected_fans: Option<u8>,
    /// The current fan information for the miner
    #[pyo3(get)]
    pub fans: Vec<FanData>,
    /// The current PDU fan information for the miner
    #[pyo3(get)]
    pub psu_fans: Vec<FanData>,
    /// The average temperature across all chips in the miner
    #[serde(serialize_with = "serialize_temperature")]
    pub average_temperature: Option<Temperature>,
    /// The environment temperature of the miner, such as air temperature or immersion fluid temperature
    #[serde(serialize_with = "serialize_temperature")]
    pub fluid_temperature: Option<Temperature>,
    /// The current power consumption of the miner
    #[serde(serialize_with = "serialize_power")]
    pub wattage: Option<Power>,
    /// The current power limit or power target of the miner
    #[serde(serialize_with = "serialize_power")]
    pub wattage_limit: Option<Power>,
    /// The current efficiency in W/TH/s (J/TH) of the miner
    #[pyo3(get)]
    pub efficiency: Option<f64>,
    /// The state of the fault/alert light on the miner
    #[pyo3(get)]
    pub light_flashing: Option<bool>,
    /// Any message on the miner, including errors
    #[pyo3(get)]
    pub messages: Vec<MinerMessage>,
    /// The total uptime of the miner's system
    #[pyo3(get)]
    pub uptime: Option<Duration>,
    /// Whether the hashing process is currently running
    #[pyo3(get)]
    pub is_mining: bool,
    /// The current pools configured on the miner
    #[pyo3(get)]
    pub pools: Vec<PoolData>,
}

#[pymethods]
impl MinerData {
    #[getter]
    pub fn mac(&self) -> Option<String> {
        self.mac.and_then(|m| Some(m.to_string()))
    }
    #[getter]
    pub fn average_temperature(&self) -> Option<f64> {
        self.average_temperature.and_then(|t| Some(t.as_celsius()))
    }
    #[getter]
    pub fn fluid_temperature(&self) -> Option<f64> {
        self.fluid_temperature.and_then(|t| Some(t.as_celsius()))
    }
    #[getter]
    pub fn wattage(&self) -> Option<f64> {
        self.wattage.and_then(|w| Some(w.as_watts()))
    }
    #[getter]
    pub fn wattage_limit(&self) -> Option<f64> {
        self.wattage_limit.and_then(|w| Some(w.as_watts()))
    }
}
