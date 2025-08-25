use pyo3::prelude::*;

pub mod miner;

#[pymodule]
pub mod asic_rs {
    #[pymodule_export]
    use super::miner::Miner;
    #[pymodule_export]
    use crate::data::MinerData;
    #[pymodule_export]
    use crate::data::device::{HashAlgorithm, MinerFirmware, MinerHardware, MinerMake, MinerModel};
    #[pymodule_export]
    use crate::miners::factory::MinerFactory;
}
