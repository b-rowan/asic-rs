pub mod v2020;

use crate::data::device::MinerModel;
use crate::miners::backends::traits::*;
use std::net::IpAddr;
use v2020::AntMinerV2020;

pub struct AntMiner;

impl AntMiner {
    pub fn new(ip: IpAddr, model: MinerModel, _: Option<semver::Version>) -> Box<dyn Miner> {
        Box::new(AntMinerV2020::new(ip, model))
    }
}
