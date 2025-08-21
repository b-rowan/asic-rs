use semver;
use std::net::IpAddr;

pub use v1::MaraV1;

use crate::data::device::MinerModel;
use crate::miners::backends::traits::{GetMinerData, MinerConstructor};

pub mod v1;

pub struct Marathon;

impl MinerConstructor for Marathon {
    #[allow(clippy::new_ret_no_self)]
    fn new(ip: IpAddr, model: MinerModel, _: Option<semver::Version>) -> Box<dyn GetMinerData> {
        Box::new(MaraV1::new(ip, model))
    }
}
