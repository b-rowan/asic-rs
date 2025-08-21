use super::miner::Miner;
use pyo3::prelude::*;

#[pymodule]
pub mod factory {
    #[allow(dead_code)]
    use std::{net::IpAddr, str::FromStr};

    use pyo3::{exceptions::PyValueError, types::PyType};
    use tokio::runtime::Runtime;

    use super::*;

    use crate::MinerFactory as RsMinerFactory;

    #[pyclass]
    #[derive(Clone)]
    struct MinerFactory {
        inner: RsMinerFactory,
    }

    #[pymethods]
    impl MinerFactory {
        #[new]
        pub fn new() -> Self {
            Self {
                inner: RsMinerFactory::new(),
            }
        }

        #[classmethod]
        pub fn with_subnet(_cls: &Bound<'_, PyType>, subnet: String) -> PyResult<Self> {
            let factory = RsMinerFactory::new().with_subnet(&subnet);
            match factory {
                Ok(inner) => Ok(Self { inner }),
                Err(e) => Err(PyValueError::new_err(e.to_string())),
            }
        }

        async fn scan(&self) -> PyResult<Vec<Miner>> {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let miners = self.inner.scan().await;
                match miners {
                    Ok(miners) => Ok(miners.into_iter().map(Miner::new).collect()),
                    Err(e) => Err(PyValueError::new_err(e.to_string())),
                }
            })
        }

        async fn get_miner(&self, ip: String) -> Option<Miner> {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let miner = self
                    .inner
                    .get_miner(IpAddr::from_str(&ip).ok()?)
                    .await
                    .ok()?;
                Some(Miner::new(miner?))
            })
        }
    }
}
