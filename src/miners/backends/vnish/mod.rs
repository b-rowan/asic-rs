use semver;
use std::net::IpAddr;

pub use v1_2_0::VnishV120;

use crate::data::device::MinerModel;
use crate::miners::backends::traits::{GetMinerData, MinerConstructor};

pub mod v1_2_0;

pub struct Vnish;

impl MinerConstructor for Vnish {
    #[allow(clippy::new_ret_no_self)]
    fn new(ip: IpAddr, model: MinerModel, _: Option<semver::Version>) -> Box<dyn GetMinerData> {
        Box::new(VnishV120::new(ip, model))
    }
}
