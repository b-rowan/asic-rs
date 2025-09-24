pub mod v25_07;

use crate::data::device::MinerModel;
use crate::miners::backends::traits::*;
use std::net::IpAddr;
use v25_07::BraiinsV2507;

pub struct Braiins;

impl MinerConstructor for Braiins {
    fn new(ip: IpAddr, model: MinerModel, _: Option<semver::Version>) -> Box<dyn Miner> {
        Box::new(BraiinsV2507::new(ip, model))
    }
}
