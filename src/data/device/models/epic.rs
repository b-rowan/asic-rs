use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
#[pyclass(module = "asic_rs", str)]
pub enum EPicModel {
    #[serde(alias = "BLOCKMINER 520i")]
    BM520i,
    #[serde(alias = "ANTMINER S19J PRO DUAL")]
    S19JProDual,
}
