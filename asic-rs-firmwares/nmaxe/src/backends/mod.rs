use std::net::IpAddr;

use asic_rs_core::traits::{
    miner::{Miner, MinerConstructor},
    model::MinerModel,
};
use v1::NMAxeV1;

pub mod v1;

pub struct NMAxe;

impl MinerConstructor for NMAxe {
    #[allow(clippy::new_ret_no_self)]
    fn new(
        ip: IpAddr,
        model: impl MinerModel,
        _version: Option<semver::Version>,
    ) -> Box<dyn Miner> {
        Box::new(NMAxeV1::new(ip, model))
    }
}
