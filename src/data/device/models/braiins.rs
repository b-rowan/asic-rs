use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
pub enum BraiinsModel {
    #[serde(alias = "BRAIINS MINI MINER BMM 100")]
    BMM100,
    #[serde(alias = "BRAIINS MINI MINER BMM 101")]
    BMM101,
}
