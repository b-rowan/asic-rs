use super::hashrate::HashRate;
use measurements::{Frequency, Temperature, Voltage};

#[derive(Debug, Clone, PartialEq)]
pub struct ChipData {
    /// The position of the chip on the board, indexed from 0
    pub position: u16,
    /// The current hashrate of the chip
    pub hashrate: Option<HashRate>,
    /// The current chip temperature
    pub temperature: Option<Temperature>,
    /// The voltage set point for this chip
    pub voltage: Option<Voltage>,
    /// The frequency set point for this chip
    pub frequency: Option<Frequency>,
    /// Whether this chip is tuned and optimizations have completed
    pub tuned: Option<bool>,
    /// Whether this chip is working and actively mining
    pub working: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardData {
    /// The board position in the miner, indexed from 0
    pub position: u8,
    /// The current hashrate of the board
    pub hashrate: Option<HashRate>,
    /// The expected or factory hashrate of the board
    pub expected_hashrate: Option<HashRate>,
    /// The board temperature, also sometimes called PCB temperature
    pub board_temperature: Option<Temperature>,
    /// The temperature of the chips at the intake, usually from the first sensor on the board
    pub intake_temperature: Option<Temperature>,
    /// The temperature of the chips at the outlet, usually from the last sensor on the board
    pub outlet_temperature: Option<Temperature>,
    /// The expected number of chips on this board
    pub expected_chips: Option<u16>,
    /// The number of working chips on this board
    pub working_chips: Option<u16>,
    /// The serial number of this board
    pub serial_number: Option<String>,
    /// Chip level information for this board
    /// May be empty, most machines do not provide this level of in depth information
    pub chips: Vec<ChipData>,
    /// The average voltage or voltage set point of this board
    pub voltage: Option<Voltage>,
    /// The average frequency or frequency set point of this board
    pub frequency: Option<Frequency>,
    /// Whether this board has been tuned and optimizations have completed
    pub tuned: Option<bool>,
    /// Whether this board is enabled and actively mining
    pub active: Option<bool>,
}
