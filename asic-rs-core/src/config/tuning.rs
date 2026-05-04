#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::miner::TuningTarget;

#[cfg_attr(feature = "python", pyclass(skip_from_py_object, module = "asic_rs"))]
#[cfg_attr(feature = "python", asic_rs_pydantic::py_pydantic_model)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    pub target: TuningTarget,
    #[cfg_attr(feature = "python", pydantic(default = None))]
    pub algorithm: Option<String>,
}

impl TuningConfig {
    pub fn new(target: TuningTarget) -> Self {
        Self {
            target,
            algorithm: None,
        }
    }

    pub fn with_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.algorithm = Some(algorithm.into());
        self
    }

    pub fn variant(&self) -> &'static str {
        match &self.target {
            TuningTarget::Power(_) => "power",
            TuningTarget::HashRate(_) => "hashrate",
            TuningTarget::MiningMode(_) => "mode",
        }
    }

    /// Target power in watts, or `None` if targeting hashrate or mining mode.
    pub fn target_watts(&self) -> Option<f64> {
        match &self.target {
            TuningTarget::Power(p) => Some(p.as_watts()),
            _ => None,
        }
    }

    /// Target hashrate, or `None` if targeting power or mining mode.
    pub fn target_hashrate(&self) -> Option<&crate::data::hashrate::HashRate> {
        match &self.target {
            TuningTarget::HashRate(hr) => Some(hr),
            _ => None,
        }
    }

    /// Target mining mode, or `None` if targeting power or hashrate.
    pub fn target_mode(&self) -> Option<crate::data::miner::MiningMode> {
        match &self.target {
            TuningTarget::MiningMode(m) => Some(*m),
            _ => None,
        }
    }

    pub fn algorithm(&self) -> Option<&str> {
        self.algorithm.as_deref()
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl TuningConfig {
    #[new]
    #[pyo3(signature = (target: "TuningTargetPower | TuningTargetHashRate | TuningTargetMode", algorithm: "HashAlgorithm | str | None" = None))]
    fn py_new(target: &Bound<'_, PyAny>, algorithm: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let mut config =
            Self::new(<TuningTarget as asic_rs_pydantic::PyPydanticType>::from_pydantic(target)?);
        if let Some(algorithm) = algorithm {
            config.algorithm = Some(asic_rs_pydantic::py_to_string(algorithm)?);
        }
        Ok(config)
    }

    #[classmethod]
    #[pyo3(signature = (watts, algorithm: "HashAlgorithm | str | None" = None))]
    fn power(
        _cls: &Bound<'_, pyo3::types::PyType>,
        watts: f64,
        algorithm: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let mut config = Self::new(TuningTarget::Power(measurements::Power::from_watts(watts)));
        if let Some(algorithm) = algorithm {
            config.algorithm = Some(asic_rs_pydantic::py_to_string(algorithm)?);
        }
        Ok(config)
    }

    #[classmethod]
    #[pyo3(signature = (hashrate, algorithm: "HashAlgorithm | str | None" = None))]
    fn hashrate(
        _cls: &Bound<'_, pyo3::types::PyType>,
        hashrate: crate::data::hashrate::HashRate,
        algorithm: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let mut config = Self::new(TuningTarget::HashRate(hashrate));
        if let Some(algorithm) = algorithm {
            config.algorithm = Some(asic_rs_pydantic::py_to_string(algorithm)?);
        }
        Ok(config)
    }

    #[classmethod]
    fn mode(_cls: &Bound<'_, pyo3::types::PyType>, mode: crate::data::miner::MiningMode) -> Self {
        Self::new(TuningTarget::MiningMode(mode))
    }

    #[getter]
    #[pyo3(name = "variant")]
    fn py_variant(&self) -> &'static str {
        self.variant()
    }

    #[getter]
    #[pyo3(name = "target")]
    fn py_target(&self) -> TuningTarget {
        self.target.clone()
    }

    /// Target power in watts, or `None` if targeting hashrate or mining mode.
    #[getter]
    #[pyo3(name = "target_watts")]
    fn py_target_watts(&self) -> Option<f64> {
        self.target_watts()
    }

    /// Target hashrate, or `None` if targeting power or mining mode.
    #[getter]
    #[pyo3(name = "target_hashrate")]
    fn py_target_hashrate(&self) -> Option<crate::data::hashrate::HashRate> {
        self.target_hashrate().cloned()
    }

    /// Target mining mode, or `None` if targeting power or hashrate.
    #[getter]
    #[pyo3(name = "target_mode")]
    fn py_target_mode(&self) -> Option<crate::data::miner::MiningMode> {
        self.target_mode()
    }

    #[getter]
    #[pyo3(name = "algorithm")]
    fn py_algorithm(&self) -> Option<&str> {
        self.algorithm()
    }
}

#[cfg(feature = "python")]
mod python_impls {
    use asic_rs_pydantic::{PyPydanticType, get_optional_field, get_required_field};
    use pyo3::{Borrowed, PyAny, PyErr, PyResult, conversion::FromPyObject, types::PyAnyMethods};

    use super::TuningConfig;
    use crate::data::miner::TuningTarget;

    impl FromPyObject<'_, '_> for TuningConfig {
        type Error = PyErr;

        fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
            let target = get_required_field(&obj, "target")?;
            let algorithm: Option<String> = get_optional_field(&obj, "algorithm")?
                .map(|value| value.extract())
                .transpose()?
                .flatten();

            Ok(TuningConfig {
                target: TuningTarget::from_pydantic(&target)?,
                algorithm,
            })
        }
    }
}
