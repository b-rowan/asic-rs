use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
