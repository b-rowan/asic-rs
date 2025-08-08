use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum BraiinsModel {
    #[serde(alias = "BRAIINS MINI MINER BMM 100")]
    BMM100,
    #[serde(alias = "BRAIINS MINI MINER BMM 101")]
    BMM101,
}
