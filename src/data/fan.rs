use crate::data::serialize;
use measurements::AngularVelocity;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serialize::serialize_angular_velocity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[pyclass(module = "asic_rs")]
pub struct FanData {
    /// The position or index of the fan as seen by the device
    /// Usually dependent on where to fan is connected to the control board
    #[pyo3(get)]
    pub position: i16,
    /// The RPM of the fan
    #[serde(serialize_with = "serialize_angular_velocity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm: Option<AngularVelocity>,
}

#[pymethods]
impl FanData {
    #[getter]
    pub fn rpm(&self) -> Option<f64> {
        self.rpm.and_then(|r| Some(r.as_rpm()))
    }
}
