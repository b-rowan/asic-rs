use std::collections::HashMap;

use anyhow::Context;
use asic_rs_core::{
    config::{
        collector::ConfigField, fan::FanConfig, scaling::ScalingConfig, tuning::TuningConfig,
    },
    data::{
        hashrate::{HashRate, HashRateUnit},
        miner::TuningTarget,
    },
};
use measurements::Power;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TuningKind {
    Power,
    Hashrate,
}

pub(crate) const LEGACY_UPDATE_AUTOTUNING_MUTATION: &str = r#"mutation ($input: AutotuningIn!) {
    bosminer {
        config {
            updateAutotuning(input: $input, apply: true) {
                ... on AutotuningOut {
                    autotuning {
                        enabled
                        mode
                        powerTarget
                        hashrateTarget
                    }
                    performanceScaling {
                        enabled
                        powerStep
                        minPowerTarget
                        hashrateStep
                        minHashrateTarget
                        shutdownEnabled
                        shutdownDuration
                    }
                }
                ... on AutotuningError {
                    message
                }
                ... on AttributeError {
                    message
                }
            }
        }
    }
}"#;

pub(crate) const LEGACY_UPDATE_TEMP_AND_FANS_MUTATION: &str = r#"mutation ($input: TempAndFansIn!) {
    bosminer {
        config {
            updateTempAndFans(input: $input, apply: true) {
                ... on TempAndFansOut {
                    tempControl {
                        mode
                        targetTemp
                    }
                    fanControl {
                        speed
                        minFans
                    }
                }
                ... on TempAndFansError {
                    message
                }
                ... on AttributeError {
                    message
                }
            }
        }
    }
}"#;

pub(crate) async fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    for chunk in bytes.chunks(64 * 1024) {
        hasher.update(chunk);
        tokio::task::yield_now().await;
    }
    format!("{:x}", hasher.finalize())
}

fn non_negative_u64(value: f64, label: &str) -> anyhow::Result<u64> {
    anyhow::ensure!(value.is_finite(), "{label} must be finite");
    anyhow::ensure!(value >= 0.0, "{label} must be non-negative");
    anyhow::ensure!(value <= u64::MAX as f64, "{label} exceeds u64::MAX");
    Ok(value.round() as u64)
}

fn power_watts(power: &Power) -> anyhow::Result<u64> {
    non_negative_u64(power.as_watts(), "power target")
}

fn hashrate_ths(hashrate: &HashRate) -> anyhow::Result<f64> {
    let ths = hashrate.clone().as_unit(HashRateUnit::TeraHash).value;
    anyhow::ensure!(ths.is_finite(), "hashrate target must be finite");
    anyhow::ensure!(ths >= 0.0, "hashrate target must be non-negative");
    Ok(ths)
}

pub(crate) fn tuning_kind(config: &TuningConfig) -> anyhow::Result<TuningKind> {
    match &config.target {
        TuningTarget::Power(_) => Ok(TuningKind::Power),
        TuningTarget::HashRate(_) => Ok(TuningKind::Hashrate),
        TuningTarget::MiningMode(_) => {
            anyhow::bail!("Braiins tuning supports power or hashrate targets, not mining modes")
        }
    }
}

fn legacy_scaling_value(config: &ScalingConfig, kind: TuningKind) -> Value {
    let mut value = Map::new();
    value.insert("enabled".to_string(), Value::Bool(true));
    match kind {
        TuningKind::Power => {
            value.insert("powerStep".to_string(), json!(config.step));
            value.insert("minPowerTarget".to_string(), json!(config.minimum));
        }
        TuningKind::Hashrate => {
            value.insert("hashrateStep".to_string(), json!(config.step as f64));
            value.insert(
                "minHashrateTarget".to_string(),
                json!(config.minimum as f64),
            );
        }
    }
    if let Some(shutdown) = config.shutdown {
        value.insert("shutdownEnabled".to_string(), json!(shutdown));
    }
    if let Some(duration) = config.shutdown_duration {
        value.insert("shutdownDuration".to_string(), json!(duration));
    }
    Value::Object(value)
}

pub(crate) fn legacy_scaling_variables(config: &ScalingConfig) -> Value {
    json!({
        "input": {
            "performanceScaling": legacy_scaling_value(config, TuningKind::Power),
        }
    })
}

pub(crate) fn legacy_tuning_variables(
    config: &TuningConfig,
    scaling_config: Option<&ScalingConfig>,
) -> anyhow::Result<Value> {
    let kind = tuning_kind(config)?;
    let mut input = Map::new();

    match &config.target {
        TuningTarget::Power(power) => {
            input.insert("mode".to_string(), json!("POWER_TARGET"));
            input.insert("powerTarget".to_string(), json!(power_watts(power)?));
        }
        TuningTarget::HashRate(hashrate) => {
            input.insert("mode".to_string(), json!("HASHRATE_TARGET"));
            input.insert("hashrateTarget".to_string(), json!(hashrate_ths(hashrate)?));
        }
        TuningTarget::MiningMode(_) => unreachable!("checked by tuning_kind"),
    }

    if let Some(scaling_config) = scaling_config {
        input.insert(
            "performanceScaling".to_string(),
            legacy_scaling_value(scaling_config, kind),
        );
    }

    Ok(json!({ "input": input }))
}

pub(crate) fn legacy_fan_variables(
    config: &FanConfig,
    include_min_fan_speed: bool,
) -> anyhow::Result<Value> {
    let mut input = Map::new();
    match config {
        FanConfig::Auto {
            target_temp,
            idle_speed,
        } => {
            anyhow::ensure!(
                target_temp.is_finite(),
                "fan target temperature must be finite"
            );
            input.insert("mode".to_string(), json!("AUTO"));
            input.insert("targetTemp".to_string(), json!(target_temp));
            if include_min_fan_speed && let Some(idle_speed) = idle_speed {
                input.insert("minFanSpeed".to_string(), json!(idle_speed));
            }
        }
        FanConfig::Manual { fan_speed } => {
            input.insert("mode".to_string(), json!("MANUAL"));
            input.insert("speed".to_string(), json!(fan_speed));
        }
    }

    Ok(json!({ "input": input }))
}

pub(crate) fn openapi_scaling_payload(
    config: &ScalingConfig,
    kind: TuningKind,
    save_action: Option<u64>,
) -> Value {
    let target = match kind {
        TuningKind::Power => json!({
            "powertarget": {
                "min_power_target": { "watt": config.minimum },
                "power_step": { "watt": config.step },
            }
        }),
        TuningKind::Hashrate => json!({
            "hashratetarget": {
                "min_hashrate_target": { "terahash_per_second": config.minimum as f64 },
                "hashrate_step": { "terahash_per_second": config.step as f64 },
            }
        }),
    };

    let mut payload = Map::new();
    payload.insert("enable".to_string(), Value::Bool(true));
    payload.insert("mode".to_string(), json!(1));
    payload.insert("target".to_string(), json!({ "target": target }));
    if let Some(save_action) = save_action {
        payload.insert("save_action".to_string(), json!(save_action));
    }
    if let Some(shutdown) = config.shutdown {
        payload.insert("enable_shutdown".to_string(), json!(shutdown));
    }
    if let Some(duration) = config.shutdown_duration {
        payload.insert(
            "shutdown_duration".to_string(),
            json!({ "hours": non_negative_u64(duration as f64, "shutdown duration").unwrap_or(0) }),
        );
    }

    Value::Object(payload)
}

pub(crate) fn openapi_tuning_payload(config: &TuningConfig) -> anyhow::Result<Value> {
    match &config.target {
        TuningTarget::Power(power) => Ok(json!({
            "tunermode": {
                "target": {
                    "powertarget": {
                        "power_target": { "watt": power_watts(power)? }
                    }
                }
            }
        })),
        TuningTarget::HashRate(hashrate) => Ok(json!({
            "tunermode": {
                "target": {
                    "hashratetarget": {
                        "hashrate_target": { "terahash_per_second": hashrate_ths(hashrate)? }
                    }
                }
            }
        })),
        TuningTarget::MiningMode(_) => {
            anyhow::bail!(
                "Braiins OpenAPI tuning supports power or hashrate targets, not mining modes"
            )
        }
    }
}

pub(crate) fn openapi_fan_payload(config: &FanConfig) -> anyhow::Result<Value> {
    match config {
        FanConfig::Auto {
            target_temp,
            idle_speed,
        } => {
            anyhow::ensure!(
                target_temp.is_finite(),
                "fan target temperature must be finite"
            );
            let mut auto = Map::new();
            auto.insert(
                "target_temperature".to_string(),
                json!({ "degree_c": target_temp }),
            );
            if let Some(idle_speed) = idle_speed {
                auto.insert("min_fan_speed".to_string(), json!(idle_speed));
            }
            Ok(json!({ "auto": auto }))
        }
        FanConfig::Manual { fan_speed } => {
            let fan_speed_ratio = (*fan_speed as f64 / 100.0).clamp(0.0, 1.0);
            Ok(json!({ "manual": { "fan_speed_ratio": fan_speed_ratio } }))
        }
    }
}

pub(crate) fn parse_legacy_scaling_config(
    data: &HashMap<ConfigField, Value>,
) -> anyhow::Result<ScalingConfig> {
    let scaling = data
        .get(&ConfigField::Scaling)
        .context("missing Braiins performance scaling config")?;
    let step = scaling
        .get("powerStep")
        .and_then(Value::as_u64)
        .context("missing Braiins powerStep")? as u32;
    let minimum = scaling
        .get("minPowerTarget")
        .and_then(Value::as_u64)
        .context("missing Braiins minPowerTarget")? as u32;

    let mut config = ScalingConfig::new(step, minimum);
    if let Some(shutdown) = scaling.get("shutdownEnabled").and_then(Value::as_bool) {
        config = config.with_shutdown(shutdown);
    }
    if let Some(duration) = scaling.get("shutdownDuration").and_then(Value::as_f64) {
        config = config.with_shutdown_duration(duration as f32);
    }
    Ok(config)
}

pub(crate) fn parse_legacy_tuning_config(
    data: &HashMap<ConfigField, Value>,
) -> anyhow::Result<TuningConfig> {
    let tuning = data
        .get(&ConfigField::Tuning)
        .context("missing Braiins autotuning config")?;
    match tuning.get("mode").and_then(Value::as_str) {
        Some("HASHRATE_TARGET") => {
            let target = tuning
                .get("hashrateTarget")
                .and_then(Value::as_f64)
                .context("missing Braiins hashrateTarget")?;
            Ok(TuningConfig::new(TuningTarget::HashRate(HashRate {
                value: target,
                unit: HashRateUnit::TeraHash,
                algo: "SHA256".to_string(),
            })))
        }
        _ => {
            let target = tuning
                .get("powerTarget")
                .and_then(Value::as_f64)
                .or_else(|| {
                    tuning
                        .get("powerTarget")
                        .and_then(Value::as_u64)
                        .map(|value| value as f64)
                })
                .context("missing Braiins powerTarget")?;
            Ok(TuningConfig::new(TuningTarget::Power(Power::from_watts(
                target,
            ))))
        }
    }
}

pub(crate) fn parse_legacy_fan_config(
    data: &HashMap<ConfigField, Value>,
) -> anyhow::Result<FanConfig> {
    let config = data
        .get(&ConfigField::Fan)
        .context("missing Braiins temperature and fan config")?;
    let temp_control = config
        .get("tempControl")
        .context("missing Braiins tempControl config")?;
    let fan_control = config
        .get("fanControl")
        .context("missing Braiins fanControl config")?;

    match temp_control.get("mode").and_then(Value::as_str) {
        Some("MANUAL") => {
            let speed = fan_control
                .get("speed")
                .and_then(Value::as_u64)
                .context("missing Braiins manual fan speed")?;
            Ok(FanConfig::manual(speed))
        }
        Some("AUTO") | Some("IMMERSION") | Some("HYDRO") | None => {
            let target_temp = temp_control
                .get("targetTemp")
                .and_then(Value::as_f64)
                .context("missing Braiins targetTemp")?;
            let idle_speed = fan_control.get("minFanSpeed").and_then(Value::as_u64);
            Ok(FanConfig::auto(target_temp, idle_speed))
        }
        Some(mode) => anyhow::bail!("unsupported Braiins fan mode {mode}"),
    }
}

pub(crate) fn parse_openapi_tuning_config(
    data: &HashMap<ConfigField, Value>,
) -> anyhow::Result<TuningConfig> {
    let tuning = data
        .get(&ConfigField::Tuning)
        .context("missing Braiins tuner state")?;

    if let Some(watts) = tuning
        .pointer("/mode_state/powertargetmodestate/current_target/watt")
        .and_then(Value::as_f64)
        .or_else(|| {
            tuning
                .pointer("/mode_state/powertargetmodestate/current_target/watt")
                .and_then(Value::as_u64)
                .map(|value| value as f64)
        })
    {
        return Ok(TuningConfig::new(TuningTarget::Power(Power::from_watts(
            watts,
        ))));
    }

    if let Some(ths) = tuning
        .pointer("/mode_state/hashratetargetmodestate/current_target/terahash_per_second")
        .and_then(Value::as_f64)
    {
        return Ok(TuningConfig::new(TuningTarget::HashRate(HashRate {
            value: ths,
            unit: HashRateUnit::TeraHash,
            algo: "SHA256".to_string(),
        })));
    }

    anyhow::bail!("missing Braiins tuner target")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_manual_fan_payload_uses_ratio() -> anyhow::Result<()> {
        assert_eq!(
            openapi_fan_payload(&FanConfig::manual(75))?,
            json!({ "manual": { "fan_speed_ratio": 0.75 } })
        );
        Ok(())
    }

    #[test]
    fn legacy_power_tuning_payload_matches_graphql_schema() -> anyhow::Result<()> {
        let tuning = TuningConfig::new(TuningTarget::Power(Power::from_watts(3500.0)));
        let scaling = ScalingConfig::new(50, 2500).with_shutdown(true);

        assert_eq!(
            legacy_tuning_variables(&tuning, Some(&scaling))?,
            json!({
                "input": {
                    "mode": "POWER_TARGET",
                    "powerTarget": 3500,
                    "performanceScaling": {
                        "enabled": true,
                        "powerStep": 50,
                        "minPowerTarget": 2500,
                        "shutdownEnabled": true,
                    },
                }
            })
        );
        Ok(())
    }
}
