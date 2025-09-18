use crate::miners::factory::MinerFactory as MinerFactory_Base;
use crate::python::miner::Miner;

use pyo3::exceptions::{PyConnectionError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

#[pyclass(module = "asic_rs")]
pub(crate) struct MinerFactory {
    inner: Arc<MinerFactory_Base>,
}

#[pymethods]
impl MinerFactory {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MinerFactory_Base::new()),
        }
    }

    #[classmethod]
    pub fn with_subnet(_cls: &Bound<'_, PyType>, subnet: String) -> PyResult<Self> {
        let factory = MinerFactory_Base::new().with_subnet(&subnet);
        match factory {
            Ok(f) => Ok(Self { inner: Arc::new(f) }),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn scan<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let miners = inner.scan().await;
            match miners {
                Ok(miners) => Ok(miners.into_iter().map(Miner::from).collect::<Vec<Miner>>()),
                Err(e) => Err(PyValueError::new_err(e.to_string())),
            }
        })
    }

    pub fn get_miner<'a>(&self, py: Python<'a>, ip: String) -> PyResult<Bound<'a, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let miner = inner.get_miner(IpAddr::from_str(&ip)?).await;
            match miner {
                Ok(Some(miner)) => Ok(Some(Miner::from(miner))),
                Ok(None) => Ok(None),
                Err(e) => Err(PyConnectionError::new_err(e.to_string())),
            }
        })
    }
}
