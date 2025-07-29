use macaddr::MacAddr;
use measurements::{AngularVelocity, Frequency, Power, Temperature, Voltage};

pub(crate) fn serialize_angular_velocity<S>(
    v: &Option<AngularVelocity>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(some) = v {
        return serializer.serialize_f64(some.as_rpm());
    }
    panic!("Cannot serialize angular velocity");
}

pub(crate) fn serialize_temperature<S>(
    t: &Option<Temperature>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(some) = t {
        return serializer.serialize_f64(some.as_celsius());
    }
    panic!("Cannot serialize temperature");
}

pub(crate) fn serialize_power<S>(p: &Option<Power>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(some) = p {
        return serializer.serialize_f64(some.as_watts());
    }
    panic!("Cannot serialize power");
}

pub(crate) fn serialize_frequency<S>(
    f: &Option<Frequency>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(some) = f {
        return serializer.serialize_f64(some.as_megahertz());
    }
    panic!("Cannot serialize frequency");
}
pub(crate) fn serialize_voltage<S>(v: &Option<Voltage>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(some) = v {
        return serializer.serialize_f64(some.as_volts());
    }
    panic!("Cannot serialize voltage");
}

pub(crate) fn serialize_macaddr<S>(v: &Option<MacAddr>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(some) = v {
        return serializer.serialize_str(&some.to_string());
    }
    panic!("Cannot serialize voltage");
}
