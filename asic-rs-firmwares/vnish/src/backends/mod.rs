use std::net::IpAddr;

use asic_rs_core::traits::{
    miner::{Miner, MinerConstructor},
    model::MinerModel,
};
use semver::Version;
pub use v1_2_0::VnishV120;
pub use v1_3_0::VnishV130;

pub mod v1_2_0;
pub mod v1_3_0;

pub struct Vnish;

impl MinerConstructor for Vnish {
    #[allow(clippy::new_ret_no_self)]
    fn new(ip: IpAddr, model: impl MinerModel, version: Option<semver::Version>) -> Box<dyn Miner> {
        match version {
            // Manual throttle (`tuning_percent`) was introduced in the 1.3.x line;
            // assumed cutoff 1.3.0 (1.3.4 verified live, 1.2.x API has no endpoint).
            Some(ref v) if *v >= Version::new(1, 3, 0) => Box::new(VnishV130::new(ip, model)),
            _ => Box::new(VnishV120::new(ip, model)),
        }
    }
}
