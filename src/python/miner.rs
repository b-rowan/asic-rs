use super::data::MinerData;
use crate::data::device::{MinerFirmware, MinerHardware, MinerMake, MinerModel};
use crate::miners::backends::traits::Miner as MinerTrait;

use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass(module = "asic_rs")]
pub(crate) struct Miner {
    inner: Arc<Box<dyn MinerTrait>>,
}

impl Miner {
    pub fn new(inner: Box<dyn MinerTrait>) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl From<Box<dyn MinerTrait>> for Miner {
    fn from(inner: Box<dyn MinerTrait>) -> Self {
        Self::new(inner)
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

    pub fn get_data<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let data = inner.get_data().await;
            Ok(MinerData::from(&data))
        })
    }
}
