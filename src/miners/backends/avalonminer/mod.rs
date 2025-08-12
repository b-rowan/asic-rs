use crate::data::device::models::avalon::AvalonMinerModel;
use crate::data::device::{MinerFirmware, MinerModel};
use crate::miners::backends::avalonminer::avalon_a::AvalonAMiner;
use crate::miners::backends::avalonminer::avalon_q::AvalonQMiner;
use crate::miners::backends::traits::GetMinerData;
use std::net::IpAddr;

pub mod avalon_a;
pub mod avalon_q;
mod shared;

pub struct AvalonMiner;

impl AvalonMiner {
    pub fn new(ip: IpAddr, model: MinerModel, firmware: MinerFirmware) -> Box<dyn GetMinerData> {
        match &model {
            MinerModel::Avalon(AvalonMinerModel::AvalonHomeQ) => {
                Box::new(AvalonQMiner::new(ip, model, firmware))
            }
            MinerModel::Avalon(_) => Box::new(AvalonAMiner::new(ip, model, firmware)),
            _ => unreachable!(),
        }
    }
}
