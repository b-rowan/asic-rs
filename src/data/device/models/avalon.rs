use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash)]
pub enum AvalonMinerModel {
    #[serde(alias = "721", alias = "AVALON 721")]
    Avalon721,
    #[serde(alias = "741", alias = "AVALON 741")]
    Avalon741,
    #[serde(alias = "761", alias = "AVALON 761")]
    Avalon761,
    #[serde(alias = "821", alias = "AVALON 821")]
    Avalon821,
    #[serde(alias = "841", alias = "AVALON 841")]
    Avalon841,
    #[serde(alias = "851", alias = "AVALON 851")]
    Avalon851,
    #[serde(alias = "921", alias = "AVALON 921")]
    Avalon921,
    #[serde(alias = "1026", alias = "AVALON 1026")]
    Avalon1026,
    #[serde(alias = "1047", alias = "AVALON 1047")]
    Avalon1047,
    #[serde(alias = "1066", alias = "AVALON 1066")]
    Avalon1066,
    #[serde(alias = "1166PRO", alias = "AVALON 1166 PRO")]
    Avalon1166Pro,
    #[serde(alias = "1126PRO", alias = "AVALON 1126 PRO")]
    Avalon1126Pro,
    #[serde(alias = "1246", alias = "AVALON 1246")]
    Avalon1246,
    #[serde(alias = "1566", alias = "AVALON 1566")]
    Avalon1566,
    #[serde(alias = "NANO3", alias = "AVALON NANO 3")]
    AvalonNano3,
    #[serde(alias = "NANO3S", alias = "AVALON NANO 3S")]
    AvalonNano3s,
    #[serde(alias = "Q")]
    AvalonHomeQ,
}
