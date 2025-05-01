use super::hashrate::HashRate;
use measurements::{Frequency, Temperature, Voltage};

#[derive(Debug, Clone, PartialEq)]
pub struct ChipData {
    pub position: u16,
    pub hashrate: Option<HashRate>,
    pub temperature: Option<Temperature>,
    pub voltage: Option<Voltage>,
    pub frequency: Option<Frequency>,
    pub tuned: Option<bool>,
    pub working: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardData {
    pub position: u8,
    pub hashrate: Option<HashRate>,
    pub expected_hashrate: Option<HashRate>,
    pub board_temperature: Option<Temperature>,
    pub intake_temperature: Option<Temperature>,
    pub outlet_temperature: Option<Temperature>,
    pub expected_chips: Option<u16>,
    pub working_chips: Option<u16>,
    pub serial_number: Option<String>,
    pub chips: Vec<ChipData>,
    pub voltage: Option<Voltage>,
    pub frequency: Option<Frequency>,
    pub tuned: Option<bool>,
    pub active: Option<bool>,
}
