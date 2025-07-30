use std::net::IpAddr;

pub use btminer_3::BTMiner3;
use semver;

use crate::data::device::{MinerFirmware, MinerModel};
use crate::miners::backends::traits::GetMinerData;

pub mod btminer_3;

pub struct BTMiner;

impl BTMiner {
    pub fn new(
        ip: IpAddr,
        model: MinerModel,
        firmware: MinerFirmware,
        version: semver::Version,
    ) -> Box<dyn GetMinerData> {
        if semver::VersionReq::parse(">=2024.11.0")
            .unwrap()
            .matches(&version)
        {
            Box::new(BTMiner3::new(ip, model, firmware))
        } else {
            panic!("Unsupported BTMiner version")
        }
    }
}
