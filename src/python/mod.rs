use pyo3::prelude::*;

pub mod factory;
pub mod miner;

#[pymodule]
pub mod asic_rs {
    #[pymodule_export]
    use super::factory::MinerFactory;
}
