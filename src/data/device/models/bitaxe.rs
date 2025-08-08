use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
pub enum BitaxeModel {
    #[serde(alias = "BM1368")]
    Supra,
    #[serde(alias = "BM1370")]
    Gamma,
    #[serde(alias = "BM1397")]
    Max,
    #[serde(alias = "BM1366")]
    Ultra,
}
