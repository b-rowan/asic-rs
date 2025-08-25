use super::hashrate::HashRate;
use super::serialize::{serialize_frequency, serialize_temperature, serialize_voltage};
use measurements::{Frequency, Temperature, Voltage};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[pyclass(module = "asic_rs")]
pub struct ChipData {
    /// The position of the chip on the board, indexed from 0
    #[pyo3(get)]
    pub position: u16,
    /// The current hashrate of the chip
    #[pyo3(get)]
    pub hashrate: Option<HashRate>,
    /// The current chip temperature
    #[serde(serialize_with = "serialize_temperature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<Temperature>,
    /// The voltage set point for this chip
    #[serde(serialize_with = "serialize_voltage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voltage: Option<Voltage>,
    /// The frequency set point for this chip
    #[serde(serialize_with = "serialize_frequency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Frequency>,
    /// Whether this chip is tuned and optimizations have completed
    #[pyo3(get)]
    pub tuned: Option<bool>,
    /// Whether this chip is working and actively mining
    #[pyo3(get)]
    pub working: Option<bool>,
}

#[pymethods]
impl ChipData {
    #[getter]
    pub fn temperature(&self) -> Option<f64> {
        self.temperature.map(|t| t.as_celsius())
    }
    #[getter]
    pub fn voltage(&self) -> Option<f64> {
        self.voltage.map(|v| v.as_volts())
    }
    #[getter]
    pub fn frequency(&self) -> Option<f64> {
        self.frequency.map(|f| f.as_megahertz())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[pyclass(module = "asic_rs")]
pub struct BoardData {
    /// The board position in the miner, indexed from 0
    #[pyo3(get)]
    pub position: u8,
    /// The current hashrate of the board
    #[pyo3(get)]
    pub hashrate: Option<HashRate>,
    /// The expected or factory hashrate of the board
    #[pyo3(get)]
    pub expected_hashrate: Option<HashRate>,
    /// The board temperature, also sometimes called PCB temperature
    #[serde(serialize_with = "serialize_temperature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_temperature: Option<Temperature>,
    /// The temperature of the chips at the intake, usually from the first sensor on the board
    #[serde(serialize_with = "serialize_temperature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intake_temperature: Option<Temperature>,
    /// The temperature of the chips at the outlet, usually from the last sensor on the board
    #[serde(serialize_with = "serialize_temperature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outlet_temperature: Option<Temperature>,
    /// The expected number of chips on this board
    #[pyo3(get)]
    pub expected_chips: Option<u16>,
    /// The number of working chips on this board
    #[pyo3(get)]
    pub working_chips: Option<u16>,
    /// The serial number of this board
    #[pyo3(get)]
    pub serial_number: Option<String>,
    /// Chip level information for this board
    /// May be empty, most machines do not provide this level of in depth information
    #[pyo3(get)]
    pub chips: Vec<ChipData>,
    /// The average voltage or voltage set point of this board
    #[serde(serialize_with = "serialize_voltage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voltage: Option<Voltage>,
    /// The average frequency or frequency set point of this board
    #[serde(serialize_with = "serialize_frequency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Frequency>,
    /// Whether this board has been tuned and optimizations have completed
    #[pyo3(get)]
    pub tuned: Option<bool>,
    /// Whether this board is enabled and actively mining
    #[pyo3(get)]
    pub active: Option<bool>,
}

#[pymethods]
impl BoardData {
    #[getter]
    pub fn board_temperature(&self) -> Option<f64> {
        self.board_temperature.map(|t| t.as_celsius())
    }
    #[getter]
    pub fn intake_temperature(&self) -> Option<f64> {
        self.intake_temperature.map(|t| t.as_celsius())
    }
    #[getter]
    pub fn outlet_temperature(&self) -> Option<f64> {
        self.outlet_temperature.map(|t| t.as_celsius())
    }
    #[getter]
    pub fn voltage(&self) -> Option<f64> {
        self.voltage.map(|v| v.as_volts())
    }
    #[getter]
    pub fn frequency(&self) -> Option<f64> {
        self.frequency.map(|f| f.as_megahertz())
    }
}
