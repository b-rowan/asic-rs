#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "python",
    pyclass(
        name = "TemperatureConfig",
        skip_from_py_object,
        get_all,
        module = "asic_rs"
    )
)]
#[cfg_attr(
    feature = "python",
    asic_rs_pydantic::py_pydantic_model(new, name = "TemperatureConfig")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configured thermal limits of the miner (°C).
///
/// Mirrors the Braiins OS layout. Fields are optional because not every
/// firmware reports every threshold (e.g. VNish exposes `minimum` and `danger`
/// but not `hot`). The target chip temperature lives on `FanConfig`.
pub struct TemperatureConfig {
    /// The temperature at which the fans run at 100%.
    pub hot: Option<f64>,
    /// The temperature at/above which the miner reboots or triggers thermal scaling.
    pub danger: Option<f64>,
    /// The minimum temperature the miner must preheat to before it will start.
    pub minimum: Option<f64>,
}

#[cfg(feature = "python")]
mod python_impls {
    use asic_rs_pydantic::get_optional_field;
    use pyo3::{Borrowed, PyAny, PyErr, PyResult, conversion::FromPyObject, types::PyAnyMethods};

    use super::TemperatureConfig;

    impl FromPyObject<'_, '_> for TemperatureConfig {
        type Error = PyErr;

        fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
            Ok(TemperatureConfig {
                hot: get_optional_field(&obj, "hot")?
                    .map(|value| value.extract())
                    .transpose()?
                    .flatten(),
                danger: get_optional_field(&obj, "danger")?
                    .map(|value| value.extract())
                    .transpose()?
                    .flatten(),
                minimum: get_optional_field(&obj, "minimum")?
                    .map(|value| value.extract())
                    .transpose()?
                    .flatten(),
            })
        }
    }
}
