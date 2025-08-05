use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvalonMinerModel {
    #[serde(alias = "Avalon 721")]
    Avalon721,
    #[serde(alias = "Avalon 741")]
    Avalon741,
    #[serde(alias = "Avalon 761")]
    Avalon761,
    #[serde(alias = "Avalon 821")]
    Avalon821,
    #[serde(alias = "Avalon 841")]
    Avalon841,
    #[serde(alias = "Avalon 851")]
    Avalon851,
    #[serde(alias = "Avalon 921")]
    Avalon921,
    #[serde(alias = "Avalon 1026")]
    Avalon1026,
    #[serde(alias = "Avalon 1047")]
    Avalon1047,
    #[serde(alias = "Avalon 1066")]
    Avalon1066,
    #[serde(alias = "Avalon 1166 Pro")]
    Avalon1166Pro,
    #[serde(alias = "Avalon 1126 Pro")]
    Avalon1126Pro,
    #[serde(alias = "Avalon 1246")]
    Avalon1246,
    #[serde(alias = "Avalon 1566")]
    Avalon1566,
    #[serde(alias = "Avalon Nano 3")]
    AvalonNano3,
    #[serde(alias = "Avalon Nano 3s")]
    AvalonNano3s,
}
