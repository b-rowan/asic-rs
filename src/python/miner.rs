use pyo3::prelude::*;

use crate::data::device::MinerHardware as RsMinerHardware;
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
    fn model(&self) -> String {
        self.inner.get_device_info().model.clone().to_string()
    }
    #[getter]
    fn make(&self) -> String {
        self.inner.get_device_info().make.clone().to_string()
    }
    #[getter]
    fn firmware(&self) -> String {
        self.inner.get_device_info().firmware.clone().to_string()
    }
    #[getter]
    fn hardware(&self) -> MinerHardware {
        MinerHardware::new(self.inner.get_device_info().hardware.clone())
    }
}

#[pyclass]
pub(crate) struct MinerHardware {
    inner: RsMinerHardware,
}

impl MinerHardware {
    pub fn new(inner: RsMinerHardware) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl MinerHardware {
    #[getter]
    fn chips(&self) -> Option<u16> {
        self.inner.chips.clone()
    }
    #[getter]
    fn boards(&self) -> Option<u8> {
        self.inner.boards.clone()
    }
    #[getter]
    fn fans(&self) -> Option<u8> {
        self.inner.fans.clone()
    }
}
