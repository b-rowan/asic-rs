use asic_rs_core::data::capabilities::{
    HashRateTuningCapabilities, PowerTuningCapabilities, TuningCapabilities,
};
use asic_rs_core::data::hashrate::{HashRate, HashRateUnit};
use asic_rs_core::data::miner::TuningTarget;
use measurements::Power;
use serde_json::Value;

/// Build [`TuningCapabilities`] from a BOS GraphQL `powerTarget` metadata object
/// (`{ default, min, max }`, all in watts). Older GraphQL BraiinsOS backends only
/// expose the power-target envelope this way.
pub(crate) fn power_target_capabilities(power_target: &Value) -> TuningCapabilities {
    let watts = |key: &str| {
        power_target
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|w| TuningTarget::from_watts(w as f64))
    };
    TuningCapabilities {
        power: Some(PowerTuningCapabilities {
            default: watts("default"),
            minimum: watts("min"),
            maximum: watts("max"),
        }),
        ..Default::default()
    }
}

/// Build [`TuningCapabilities`] from a BOS+ REST `tuner_constraints` object
/// (`power_target` as `PowerConstraints` in watts, `hashrate_target` as
/// `HashrateConstraints` in TH/s). Used by the REST BraiinsOS backends
/// (25.07 and newer).
pub(crate) fn tuner_constraints_capabilities(tuner: &Value) -> TuningCapabilities {
    let power = tuner.get("power_target").map(|target| {
        let watts = |key: &str| {
            target
                .pointer(&format!("/{key}/watt"))
                .and_then(|v| v.as_i64())
                .map(|w| TuningTarget::from_watts(w as f64))
        };
        PowerTuningCapabilities {
            default: watts("default"),
            minimum: watts("min"),
            maximum: watts("max"),
        }
    });

    let hashrate = tuner.get("hashrate_target").map(|target| {
        let ths = |key: &str| {
            target
                .pointer(&format!("/{key}/terahash_per_second"))
                .and_then(|v| v.as_f64())
                .map(|value| {
                    TuningTarget::HashRate(HashRate {
                        value,
                        unit: HashRateUnit::TeraHash,
                        algo: "SHA256".to_string(),
                    })
                })
        };
        HashRateTuningCapabilities {
            default: ths("default"),
            minimum: ths("min"),
            maximum: ths("max"),
        }
    });

    TuningCapabilities {
        power,
        hashrate,
        presets: None,
    }
}

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
