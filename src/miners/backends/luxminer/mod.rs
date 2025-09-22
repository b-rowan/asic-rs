use crate::data::device::MinerModel;
use crate::miners::backends::traits::GetMinerData;
use std::net::IpAddr;
use v1::LuxMinerV1;

pub mod v1;

pub struct LuxMiner;

impl LuxMiner {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ip: IpAddr, model: MinerModel, _: Option<semver::Version>) -> Box<dyn GetMinerData> {
        Box::new(LuxMinerV1::new(ip, model))
    }
}
