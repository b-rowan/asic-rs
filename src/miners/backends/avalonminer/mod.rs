use semver;
use std::net::IpAddr;

pub use avalon_a::AvalonAMiner;
pub use avalon_q::AvalonQMiner;

use crate::data::device::MinerModel;
use crate::data::device::models::avalon::AvalonMinerModel;
use crate::miners::backends::traits::GetMinerData;

pub mod avalon_a;
pub mod avalon_q;

pub struct AvalonMiner;

impl AvalonMiner {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ip: IpAddr, model: MinerModel, _: Option<semver::Version>) -> Box<dyn GetMinerData> {
        match &model {
            MinerModel::AvalonMiner(AvalonMinerModel::AvalonHomeQ) => {
                Box::new(AvalonQMiner::new(ip, model))
            }
            MinerModel::AvalonMiner(_) => Box::new(AvalonAMiner::new(ip, model)),
            _ => unreachable!(),
        }
    }
}
