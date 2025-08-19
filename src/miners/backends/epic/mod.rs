use semver;
use std::net::IpAddr;

pub use v1::PowerPlayV1;

use crate::data::device::{MinerMake, MinerModel};
use crate::miners::backends::traits::GetMinerData;

pub mod v1;

pub struct PowerPlay;

impl PowerPlay {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        ip: IpAddr,
        make: MinerMake,
        model: MinerModel,
        _: Option<semver::Version>,
    ) -> Box<dyn GetMinerData> {
        Box::new(PowerPlayV1::new(ip, make, model))
    }
}
