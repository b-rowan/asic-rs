//! Tuning capability metadata reported by a miner's firmware.
//!
//! Where [`MinerData::tuning_target`](super::miner::MinerData::tuning_target)
//! describes the *current* tuning setpoint, these types describe the *envelope*
//! the firmware allows: the factory default plus the accepted range (for power
//! and hashrate tuning) or the set of selectable presets. Every field is
//! optional — a backend fills in only what its firmware actually exposes.

#[cfg(feature = "python")]
use pyo3::pyclass;
use serde::{Deserialize, Serialize};

use super::miner::TuningTarget;

/// Factory tuning envelope for power-target tuning.
#[cfg_attr(feature = "python", pyclass(from_py_object, module = "asic_rs"))]
#[cfg_attr(feature = "python", asic_rs_pydantic::py_pydantic_model(getters))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PowerTuningCapabilities {
    /// The factory default power target, i.e. the value tuning starts from.
    pub default: Option<TuningTarget>,
    /// The lowest power target the miner accepts.
    pub minimum: Option<TuningTarget>,
    /// The highest power target the miner accepts.
    pub maximum: Option<TuningTarget>,
}

/// Factory tuning envelope for hashrate-target tuning.
#[cfg_attr(feature = "python", pyclass(from_py_object, module = "asic_rs"))]
#[cfg_attr(feature = "python", asic_rs_pydantic::py_pydantic_model(getters))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HashRateTuningCapabilities {
    /// The factory default hashrate target, i.e. the value tuning starts from.
    pub default: Option<TuningTarget>,
    /// The lowest hashrate target the miner accepts.
    pub minimum: Option<TuningTarget>,
    /// The highest hashrate target the miner accepts.
    pub maximum: Option<TuningTarget>,
}

/// Factory tuning envelope for preset / mining-mode tuning.
#[cfg_attr(feature = "python", pyclass(from_py_object, module = "asic_rs"))]
#[cfg_attr(feature = "python", asic_rs_pydantic::py_pydantic_model(getters))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PresetTuningCapabilities {
    /// The factory default preset, when one is marked as default.
    pub default: Option<TuningTarget>,
    /// The presets the firmware offers to select between.
    pub presets: Vec<TuningTarget>,
}

/// The tuning envelopes a miner exposes, grouped by tuning domain.
///
/// A backend populates only the domains its firmware supports; the rest stay
/// `None`. This keeps a single, extensible home for "what can this miner be
/// tuned to" instead of a flat list of per-aspect fields.
#[cfg_attr(feature = "python", pyclass(from_py_object, module = "asic_rs"))]
#[cfg_attr(feature = "python", asic_rs_pydantic::py_pydantic_model(getters))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TuningCapabilities {
    /// Power-target tuning envelope, when the firmware tunes by power.
    pub power: Option<PowerTuningCapabilities>,
    /// Hashrate-target tuning envelope, when the firmware tunes by hashrate.
    pub hashrate: Option<HashRateTuningCapabilities>,
    /// Preset / mining-mode tuning envelope, when the firmware tunes by preset.
    pub presets: Option<PresetTuningCapabilities>,
}
