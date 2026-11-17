use std::{fmt::Display, net::IpAddr};

use asic_rs_core::{
    data::command::DiscoveryCommand,
    discovery::HTTP_WEB_ROOT,
    errors::ModelSelectionError,
    traits::{
        discovery::DiscoveryCommands,
        entry::FirmwareEntry,
        firmware::MinerFirmware,
        identification::{FirmwareIdentification, WebResponse},
        make::MinerMake,
        miner::{Miner, MinerAuth, MinerConstructor},
    },
    util,
};
use asic_rs_makes_nmaxe::make::NMAxeMake;
use async_trait::async_trait;

#[derive(Default, Debug)]
pub struct NMAxeFirmware {}

impl Display for NMAxeFirmware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NMAxe Stock")
    }
}

impl DiscoveryCommands for NMAxeFirmware {
    fn get_discovery_commands(&self) -> Vec<DiscoveryCommand> {
        vec![HTTP_WEB_ROOT]
    }
}

fn parse_model_from_info(
    info: &serde_json::Value,
) -> Result<asic_rs_makes_nmaxe::models::NMAxeModel, ModelSelectionError> {
    // hwModel is "NMQAxe++" for the base, revision, and Nexus boards.
    // displayName is the only distinct option.
    // Use hwModel for older firmware that does not provide a display name.
    let model = info
        .pointer("/identity/displayName")
        .and_then(serde_json::Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .or_else(|| {
            info.pointer("/identity/hwModel")
                .and_then(serde_json::Value::as_str)
        })
        .ok_or(ModelSelectionError::UnexpectedModelResponse)?;
    NMAxeMake::parse_model(model.to_string())
}

#[async_trait]
impl MinerFirmware for NMAxeFirmware {
    type Model = asic_rs_makes_nmaxe::models::NMAxeModel;

    async fn get_model(ip: IpAddr) -> Result<Self::Model, ModelSelectionError> {
        let (text, _, _) = util::send_web_command(&ip, "/api/system/info")
            .await
            .ok_or(ModelSelectionError::NoModelResponse)?;

        let json_data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|_| ModelSelectionError::UnexpectedModelResponse)?;

        parse_model_from_info(&json_data)
    }

    async fn get_version(ip: IpAddr) -> Option<semver::Version> {
        let (text, _, _) = util::send_web_command(&ip, "/api/system/info").await?;
        let json_data: serde_json::Value = serde_json::from_str(&text).ok()?;
        let version_str = json_data["identity"]["fwVersion"]
            .as_str()?
            .trim_start_matches('v');
        semver::Version::parse(version_str).ok()
    }
}

impl FirmwareIdentification for NMAxeFirmware {
    fn identify_web(&self, response: &WebResponse<'_>) -> bool {
        if response.status != 200 {
            return false;
        }
        let Ok(data) = serde_json::from_str::<serde_json::Value>(response.body) else {
            // Some NMAxe releases retain the generic AxeOS page title.
            return response.body.contains("NMAxe")
                || response.body.contains("NMQAxe")
                || response.body.contains("AxeOS");
        };
        data.pointer("/identity/hwModel")
            .and_then(|model| model.as_str())
            .is_some_and(|model| model.starts_with("NMAxe") || model.starts_with("NMQAxe"))
    }

    fn is_stock(&self) -> bool {
        true
    }
}

#[async_trait]
impl FirmwareEntry for NMAxeFirmware {
    fn secondary_web_path(&self) -> Option<&'static str> {
        Some("/api/system/info")
    }

    async fn build_miner_from_web_response(
        &self,
        ip: IpAddr,
        auth: Option<&MinerAuth>,
        response: &WebResponse<'_>,
    ) -> Result<Box<dyn Miner>, ModelSelectionError> {
        if response.status != 200 {
            return Err(ModelSelectionError::UnexpectedModelResponse);
        }
        let info: serde_json::Value = serde_json::from_str(response.body)
            .map_err(|_| ModelSelectionError::UnexpectedModelResponse)?;
        let model = parse_model_from_info(&info)?;
        let version = info["identity"]["fwVersion"]
            .as_str()
            .and_then(|version| semver::Version::parse(version.trim_start_matches('v')).ok());
        let mut miner = crate::backends::NMAxe::new(ip, model, version);
        if let Some(auth) = auth {
            miner.set_auth(auth.clone());
        }
        Ok(miner)
    }

    async fn build_miner(
        &self,
        ip: IpAddr,
        auth: Option<&MinerAuth>,
    ) -> Result<Box<dyn Miner>, ModelSelectionError> {
        let model = NMAxeFirmware::get_model(ip).await?;
        let version = NMAxeFirmware::get_version(ip).await;
        let mut miner = crate::backends::NMAxe::new(ip, model, version);
        if let Some(auth) = auth {
            miner.set_auth(auth.clone());
        }
        Ok(miner)
    }
}

#[cfg(test)]
mod model_tests {
    use super::*;
    use asic_rs_makes_nmaxe::models::NMAxeModel;
    use serde_json::json;

    #[test]
    fn display_name_selects_the_hardware_variant() -> Result<(), ModelSelectionError> {
        for (display, expected) in [
            ("NMQAxe++", NMAxeModel::NMQAxePlusPlus),
            ("NMQAxe++Rev6.1", NMAxeModel::NMQAxePlusPlus),
            ("NMQAxe++Rev8.1", NMAxeModel::NMQAxePlusPlusRev81),
            ("NMQAxe++Nexus", NMAxeModel::NMQAxePlusPlusNexus),
        ] {
            let info = json!({"identity": {"hwModel": "NMQAxe++", "displayName": display}});
            assert_eq!(parse_model_from_info(&info)?, expected);
        }
        for info in [
            json!({"identity": {"hwModel": "NMAxe"}}),
            json!({"identity": {"hwModel": "NMAxe", "displayName": ""}}),
            json!({"identity": {"hwModel": "NMAxe", "displayName": null}}),
        ] {
            assert_eq!(parse_model_from_info(&info)?, NMAxeModel::NMAxe);
        }
        assert_eq!(
            parse_model_from_info(
                &json!({"identity": {"hwModel": "NMQAxe++", "displayName": "NMQAxe++Future"}})
            )?,
            NMAxeModel::Unknown("NMQAxe++Future".into())
        );
        assert!(parse_model_from_info(&json!({})).is_err());
        Ok(())
    }
}
