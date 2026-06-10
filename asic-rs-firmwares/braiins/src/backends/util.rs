use asic_rs_core::data::hashrate::{HashRate, HashRateUnit};
use asic_rs_core::data::miner::TuningTarget;
use measurements::Power;
use serde_json::Value;

pub(crate) fn parse_configured_tuning_target(value: &Value) -> Option<TuningTarget> {
    parse_tagged_tuning_target(value, "configured")
}

pub(crate) fn parse_scaled_tuning_target(value: &Value) -> Option<TuningTarget> {
    parse_tagged_tuning_target(value, "scaled")
}

fn parse_tagged_tuning_target(value: &Value, prefix: &str) -> Option<TuningTarget> {
    let power_key = format!("{prefix}_power");
    let hashrate_key = format!("{prefix}_hashrate");

    let power = value
        .get(&power_key)
        .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
        .map(Power::from_watts)
        .map(TuningTarget::Power);

    let hashrate = value
        .get(&hashrate_key)
        .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
        .map(|value| {
            TuningTarget::HashRate(HashRate {
                value,
                unit: HashRateUnit::TeraHash,
                algo: "SHA256".to_string(),
            })
        });

    match (
        value.get("mode").and_then(Value::as_str),
        value.get("mode").and_then(Value::as_i64),
    ) {
        (Some("HASHRATE_TARGET"), _) | (_, Some(2)) => hashrate.or(power),
        (Some("POWER_TARGET"), _) | (_, Some(1)) => power.or(hashrate),
        _ => power.or(hashrate),
    }
}
