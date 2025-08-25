use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::device::{MinerFirmware, MinerHardware, MinerMake, MinerModel};
use crate::data::miner::MinerData;
use crate::miners::backends::traits::GetMinerData;

#[pyclass]
pub(crate) struct Miner {
    inner: Box<dyn GetMinerData>,
}

impl Miner {
    pub fn new(inner: Box<dyn GetMinerData>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl Miner {
    fn __repr__(&self) -> String {
        format!("{} {} ({})", self.make(), self.model(), self.firmware())
    }

    #[getter]
    fn model(&self) -> MinerModel {
        self.inner.get_device_info().model
    }
    #[getter]
    fn make(&self) -> MinerMake {
        self.inner.get_device_info().make
    }
    #[getter]
    fn firmware(&self) -> MinerFirmware {
        self.inner.get_device_info().firmware
    }
    #[getter]
    fn hardware(&self) -> MinerHardware {
        self.inner.get_device_info().hardware
    }

    async fn get_data(&self) -> MinerData {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { self.inner.get_data().await })
    }
}
