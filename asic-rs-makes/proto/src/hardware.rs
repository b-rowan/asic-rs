use std::{fmt::Display, str::FromStr};

use asic_rs_core::data::{board::MinerControlBoard, device::MinerHardware};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::Display as StrumDisplay;

use crate::models::ProtoModel;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, StrumDisplay)]
pub enum ProtoHashboardKind {
    CpuSimulated,
    B2,
    B3a,
    B3b,
    B3bSim,
    B4,
    B4Sim,
    #[strum(to_string = "{0}")]
    Unknown(String),
}

impl ProtoHashboardKind {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "CpuSimulated" => Self::CpuSimulated,
            "B2" => Self::B2,
            "B3a" => Self::B3a,
            "B3b" => Self::B3b,
            "B3bSim" => Self::B3bSim,
            "B4" => Self::B4,
            "B4Sim" => Self::B4Sim,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl FromStr for ProtoHashboardKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, StrumDisplay)]
pub enum ProtoMiningAsic {
    BZM,
    MC1,
    MC2,
    MC3,
    #[strum(to_string = "{0}")]
    Unknown(String),
}

impl ProtoMiningAsic {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "BZM" => Self::BZM,
            "MC1" => Self::MC1,
            "MC2" => Self::MC2,
            "MC3" => Self::MC3,
            other => Self::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, StrumDisplay)]
pub enum ProtoControlBoard {
    C1,
    C2,
    C3,
    #[strum(to_string = "{0}")]
    Unknown(String),
}

impl ProtoControlBoard {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_uppercase().as_str() {
            "C1" => Self::C1,
            "C2" => Self::C2,
            "C3" => Self::C3,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl From<ProtoControlBoard> for MinerControlBoard {
    fn from(cb: ProtoControlBoard) -> Self {
        match cb {
            ProtoControlBoard::Unknown(name) => MinerControlBoard::unknown(name),
            known => MinerControlBoard::known(known.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ProtoHashboardConfig {
    pub position: u8,
    pub board: ProtoHashboardKind,
    pub mining_asic: Option<ProtoMiningAsic>,
    pub chips: Option<u16>,
    pub serial_number: Option<String>,
}

impl ProtoHashboardConfig {
    pub fn new(position: u8, board: ProtoHashboardKind, chips: Option<u16>) -> Self {
        Self {
            position,
            board,
            mining_asic: None,
            chips,
            serial_number: None,
        }
    }

    pub fn with_mining_asic(mut self, mining_asic: ProtoMiningAsic) -> Self {
        self.mining_asic = Some(mining_asic);
        self
    }

    pub fn with_serial_number(mut self, serial_number: impl Into<String>) -> Self {
        self.serial_number = Some(serial_number.into());
        self
    }

    fn from_mdk_hashboard(value: &Value, fallback_position: u8) -> Self {
        let position = value
            .get("slot")
            .and_then(Value::as_u64)
            .and_then(|slot| slot.checked_sub(1))
            .or_else(|| value.get("port").and_then(Value::as_u64))
            .and_then(|position| u8::try_from(position).ok())
            .unwrap_or(fallback_position);

        let board = value
            .get("board")
            .and_then(Value::as_str)
            .map(ProtoHashboardKind::parse)
            .unwrap_or_else(|| ProtoHashboardKind::Unknown("Unknown".to_string()));

        let mining_asic = value
            .get("mining_asic")
            .and_then(Value::as_str)
            .map(ProtoMiningAsic::parse);

        let chips = value
            .get("mining_asic_count")
            .and_then(Value::as_u64)
            .and_then(|count| u16::try_from(count).ok());

        let serial_number = value
            .get("hb_sn")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        Self {
            position,
            board,
            mining_asic,
            chips,
            serial_number,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Default)]
pub struct ProtoHardwareConfig {
    pub hashboards: Vec<ProtoHashboardConfig>,
    pub fans: Option<u8>,
    pub control_board: Option<ProtoControlBoard>,
}

impl ProtoHardwareConfig {
    pub fn new(hashboards: Vec<ProtoHashboardConfig>, fans: Option<u8>) -> Self {
        Self {
            hashboards,
            fans,
            control_board: None,
        }
    }

    pub fn from_counts(boards: u8, chips_per_board: Option<u16>, fans: Option<u8>) -> Self {
        let hashboards = (0..boards)
            .map(|position| {
                ProtoHashboardConfig::new(
                    position,
                    ProtoHashboardKind::Unknown("Unknown".to_string()),
                    chips_per_board,
                )
            })
            .collect();
        Self::new(hashboards, fans)
    }

    pub fn with_control_board(mut self, control_board: ProtoControlBoard) -> Self {
        self.control_board = Some(control_board);
        self
    }

    pub fn from_mdk_hardware_info(value: &Value) -> Self {
        let hardware = value.get("hardware-info").unwrap_or(value);

        let hashboards = hardware
            .get("hashboards-info")
            .and_then(Value::as_array)
            .map(|boards| {
                boards
                    .iter()
                    .enumerate()
                    .map(|(idx, board)| {
                        let fallback = u8::try_from(idx).unwrap_or(u8::MAX);
                        ProtoHashboardConfig::from_mdk_hashboard(board, fallback)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let fans = hardware
            .get("fans-info")
            .and_then(Value::as_array)
            .and_then(|fans| u8::try_from(fans.len()).ok());

        let control_board = hardware
            .get("cb-info")
            .and_then(|cb| {
                cb.get("machine_name")
                    .and_then(Value::as_str)
                    .or_else(|| cb.get("board_id").and_then(Value::as_str))
            })
            .map(ProtoControlBoard::parse);

        Self {
            hashboards,
            fans,
            control_board,
        }
    }

    pub fn board_count(&self) -> Option<u8> {
        if self.hashboards.is_empty() {
            None
        } else {
            u8::try_from(self.hashboards.len()).ok()
        }
    }

    pub fn uniform_chips_per_board(&self) -> Option<u16> {
        let mut boards = self.hashboards.iter();
        let first = boards.next()?.chips?;
        if boards.all(|board| board.chips == Some(first)) {
            Some(first)
        } else {
            None
        }
    }

    pub fn chips_for_position(&self, position: u8) -> Option<u16> {
        self.hashboards
            .iter()
            .find(|board| board.position == position)
            .and_then(|board| board.chips)
    }
}

impl Display for ProtoHardwareConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let boards = self.board_count().unwrap_or(0);
        let fans = self
            .fans
            .map(|fans| fans.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        write!(f, "{boards} hashboards, {fans} fans")
    }
}

impl From<ProtoModel> for MinerHardware {
    fn from(model: ProtoModel) -> Self {
        Self {
            chips: model.hardware().uniform_chips_per_board(),
            fans: model.hardware().fans,
            boards: model.hardware().board_count(),
        }
    }
}
