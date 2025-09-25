use semver;
use std::net::IpAddr;

pub use v2_0_0::Bitaxe200;
pub use v2_9_0::Bitaxe290;

use crate::data::device::MinerModel;
use crate::miners::backends::traits::*;

pub mod v2_0_0;
pub mod v2_9_0;

pub struct Bitaxe;

impl MinerConstructor for Bitaxe {
    #[allow(clippy::new_ret_no_self)]
    fn new(ip: IpAddr, model: MinerModel, version: Option<semver::Version>) -> Box<dyn Miner> {
        if let Some(v) = version {
            if semver::VersionReq::parse(">=2.0.0, <2.9.0")
                .unwrap()
                .matches(&v)
            {
                Box::new(Bitaxe200::new(ip, model))
            } else if semver::VersionReq::parse(">=2.9.0").unwrap().matches(&v) {
                Box::new(Bitaxe290::new(ip, model))
            } else {
                panic!("Unsupported Bitaxe version")
            }
        } else {
            panic!("Unsupported Bitaxe version")
        }
    }
}
