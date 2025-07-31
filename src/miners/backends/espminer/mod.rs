use std::net::IpAddr;

use semver;
pub use v2_0_0::ESPMiner200;
pub use v2_9_0::ESPMiner290;

use crate::data::device::{MinerFirmware, MinerModel};
use crate::miners::backends::traits::GetMinerData;

pub mod v2_0_0;
pub mod v2_9_0;

pub struct ESPMiner;

impl ESPMiner {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        ip: IpAddr,
        model: MinerModel,
        firmware: MinerFirmware,
        version: semver::Version,
    ) -> Box<dyn GetMinerData> {
        if semver::VersionReq::parse(">=2.0.0, <2.9.0")
            .unwrap()
            .matches(&version)
        {
            Box::new(ESPMiner200::new(ip, model, firmware))
        } else if semver::VersionReq::parse(">=2.9.0")
            .unwrap()
            .matches(&version)
        {
            Box::new(ESPMiner290::new(ip, model, firmware))
        } else {
            panic!("Unsupported ESPMiner version")
        }
    }
}
