use std::{fmt::Display, net::IpAddr};

use asic_rs_core::{
    data::command::MinerCommand,
    errors::ModelSelectionError,
    traits::{
        discovery::DiscoveryCommands,
        entry::FirmwareEntry,
        firmware::MinerFirmware,
        identification::{FirmwareIdentification, WebResponse},
        miner::{Miner, MinerAuth, MinerConstructor},
        model::MinerModel,
    },
    util,
};
use asic_rs_makes_proto::{hardware::ProtoHardwareConfig, make::ProtoMake};
use async_trait::async_trait;

#[derive(Default, Debug)]
pub struct ProtoFirmware {}

impl Display for ProtoFirmware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Proto MDK")
    }
}

const WEB_SYSTEM: MinerCommand = MinerCommand::WebAPI {
    command: "/api/v1/system",
    parameters: None,
};

impl DiscoveryCommands for ProtoFirmware {
    fn get_discovery_commands(&self) -> Vec<MinerCommand> {
        vec![WEB_SYSTEM]
    }
}

fn parse_semver_like(version: &str) -> Option<semver::Version> {
    let trimmed = version.trim().trim_start_matches('v');
    semver::Version::parse(trimmed)
        .or_else(|_| semver::Version::parse(&format!("{trimmed}.0")))
        .or_else(|_| semver::Version::parse(&format!("{trimmed}.0.0")))
        .ok()
}

async fn get_json(ip: IpAddr, command: &'static str) -> Option<serde_json::Value> {
    let (body, _, status) = util::send_web_command(&ip, command).await?;
    if !status.is_success() {
        return None;
    }
    serde_json::from_str(&body).ok()
}

#[async_trait]
impl MinerFirmware for ProtoFirmware {
    async fn get_model(ip: IpAddr) -> Result<impl MinerModel, ModelSelectionError> {
        let system = get_json(ip, "/api/v1/system")
            .await
            .ok_or(ModelSelectionError::NoModelResponse)?;
        let system_info = system
            .get("system-info")
            .ok_or(ModelSelectionError::UnexpectedModelResponse)?;

        let model_name = system_info
            .get("product_name")
            .and_then(serde_json::Value::as_str)
            .or_else(|| system_info.get("model").and_then(serde_json::Value::as_str))
            .unwrap_or("Rig")
            .to_string();

        let hardware = get_json(ip, "/api/v1/hardware")
            .await
            .map(|value| ProtoHardwareConfig::from_mdk_hardware_info(&value))
            .unwrap_or_default();

        Ok(ProtoMake::configured_model(model_name, hardware))
    }

    async fn get_version(ip: IpAddr) -> Option<semver::Version> {
        let system = get_json(ip, "/api/v1/system").await?;
        let system_info = system.get("system-info")?;

        for pointer in [
            "/mining_driver_sw/version",
            "/web_server/version",
            "/web_dashboard/version",
            "/hashboard_firmware/version",
            "/pool_interface_sw/version",
            "/os/version",
        ] {
            if let Some(version) = system_info
                .pointer(pointer)
                .and_then(serde_json::Value::as_str)
                && let Some(parsed) = parse_semver_like(version)
            {
                return Some(parsed);
            }
        }

        None
    }
}

impl FirmwareIdentification for ProtoFirmware {
    fn identify_web(&self, response: &WebResponse<'_>) -> bool {
        if !response.status.to_string().starts_with('2') {
            return false;
        }

        let Ok(value) = serde_json::from_str::<serde_json::Value>(response.body) else {
            return false;
        };

        value
            .get("system-info")
            .and_then(|system| {
                system
                    .get("manufacturer")
                    .and_then(serde_json::Value::as_str)
                    .or_else(|| {
                        system
                            .get("product_name")
                            .and_then(serde_json::Value::as_str)
                    })
            })
            .is_some_and(|name| name.to_ascii_lowercase().contains("proto"))
    }

    fn is_stock(&self) -> bool {
        true
    }
}

#[async_trait]
impl FirmwareEntry for ProtoFirmware {
    async fn build_miner(
        &self,
        ip: IpAddr,
        auth: Option<&MinerAuth>,
    ) -> Result<Box<dyn Miner>, ModelSelectionError> {
        let model = ProtoFirmware::get_model(ip).await?;
        let version = ProtoFirmware::get_version(ip).await;
        let mut miner = crate::backends::Proto::new(ip, model, version);
        if let Some(auth) = auth {
            miner.set_auth(auth.clone());
        }
        Ok(miner)
    }
}
