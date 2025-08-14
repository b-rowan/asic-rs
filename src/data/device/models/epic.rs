use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
pub enum EPicModel {
    #[serde(alias = "BLOCKMINER 520i")]
    BM520i,
    #[serde(alias = "ANTMINER S19J PRO DUAL")]
    S19JProDual,
}
