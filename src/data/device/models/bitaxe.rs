use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
#[pyclass(module = "asic_rs", str)]
pub enum BitAxeModel {
    #[serde(alias = "BM1368")]
    Supra,
    #[serde(alias = "BM1370")]
    Gamma,
    #[serde(alias = "BM1397")]
    Max,
    #[serde(alias = "BM1366")]
    Ultra,
}
