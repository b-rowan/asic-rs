use crate::data::device::models::MinerModelFactory;
use crate::data::device::{MinerFirmware, MinerMake, MinerModel};
use crate::miners::factory::model::whatsminer::{get_model_whatsminer_v2, get_model_whatsminer_v3};
use crate::miners::util;
use chrono::{Datelike, NaiveDateTime};
use diqwest::WithDigestAuth;
use reqwest::{Client, Response};
use semver;
use std::net::IpAddr;

pub mod whatsminer;

pub(crate) async fn get_model_vnish(ip: IpAddr) -> Option<MinerModel> {
    let response: Option<Response> = Client::new()
        .get(format!("http://{ip}/api/v1/info"))
        .send()
        .await
        .ok();

    match response {
        Some(data) => {
            let json_data = data.json::<serde_json::Value>().await.ok()?;
            let model = json_data["model"].as_str().unwrap_or("");

            // VnishOS typically runs on AntMiner hardware
            let mut factory = MinerModelFactory::new();
            factory.with_make(MinerMake::AntMiner).parse_model(model)
        }
        None => None,
    }
}

pub(crate) async fn get_version_vnish(ip: IpAddr) -> Option<semver::Version> {
    let response: Option<Response> = Client::new()
        .get(format!("http://{ip}/api/v1/info"))
        .send()
        .await
        .ok();

    match response {
        Some(data) => {
            let json_data = data.json::<serde_json::Value>().await.ok()?;
            let fw_version = json_data["fw_version"].as_str().unwrap_or("");

            // Try parsing directly first
            if let Ok(version) = semver::Version::parse(fw_version) {
                return Some(version);
            }

            // If direct parsing fails, try adding .0 for patch version
            let normalized_version = format!("{fw_version}.0");
            semver::Version::parse(&normalized_version).ok()
        }
        None => None,
    }
}

pub(crate) async fn get_model_epic(ip: IpAddr) -> Option<MinerModel> {
    let response: Option<Response> = Client::new()
        .get(format!("http://{ip}:4028/capabilities"))
        .send()
        .await
        .ok();

    match response {
        Some(data) => {
            let json_data = data.json::<serde_json::Value>().await.ok()?;
            let model = json_data["Model"].as_str().unwrap_or("").to_uppercase();

            MinerModelFactory::new()
                .with_firmware(MinerFirmware::EPic)
                .parse_model(&model)
        }
        None => None,
    }
}
pub(crate) async fn get_version_epic(ip: IpAddr) -> Option<semver::Version> {
    let response: Option<Response> = Client::new()
        .get(format!("http://{ip}:4028/summary"))
        .send()
        .await
        .ok();

    match response {
        Some(data) => {
            let json_data = data.json::<serde_json::Value>().await.ok()?;
            let fw_version = json_data["Software"]
                .as_str()
                .unwrap_or("")
                .split(" ")
                .last()?
                .strip_prefix("v")?;
            semver::Version::parse(fw_version).ok()
        }
        None => None,
    }
}

pub(crate) async fn get_model_antminer(ip: IpAddr) -> Option<MinerModel> {
    let response: Option<Response> = Client::new()
        .get(format!("http://{ip}/cgi-bin/get_system_info.cgi"))
        .send_with_digest_auth("root", "root")
        .await
        .ok();
    match response {
        Some(data) => {
            let json_data = data.json::<serde_json::Value>().await.ok()?;
            let model = json_data["minertype"].as_str().unwrap_or("").to_uppercase();

            MinerModelFactory::new()
                .with_make(MinerMake::AntMiner)
                .parse_model(&model)
        }
        None => None,
    }
}

pub(crate) async fn get_version_antminer(ip: IpAddr) -> Option<semver::Version> {
    let response: Option<Response> = Client::new()
        .get(format!("http://{ip}/cgi-bin/summary.cgi"))
        .send_with_digest_auth("root", "root")
        .await
        .ok();
    match response {
        Some(data) => {
            let json_data = data.json::<serde_json::Value>().await.ok()?;
            let fw_version = json_data["INFO"]["CompileTime"].as_str().unwrap_or("");

            let cleaned: String = {
                let mut parts: Vec<&str> = fw_version.split_whitespace().collect();
                parts.remove(4); // remove time zone
                parts.join(" ")
            };

            let dt = NaiveDateTime::parse_from_str(&cleaned, "%a %b %e %H:%M:%S %Y").ok()?;

            let version =
                semver::Version::new(dt.year() as u64, dt.month() as u64, dt.day() as u64);

            Some(version)
        }
        None => None,
    }
}

pub(crate) async fn get_model_whatsminer(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "get_version").await;

    match response {
        Some(json_data) => {
            let fw_version: Option<&str> = json_data["Msg"]["fw_ver"].as_str();
            fw_version?;

            let fw_version = fw_version.unwrap();

            // Parse the firmware version format: YYYYMMDD.XX.REL
            // Extract the date components
            if fw_version.len() < 8 {
                return None;
            }

            let date_part = &fw_version[..8];
            if let (Ok(year), Ok(month), Ok(day)) = (
                date_part[..4].parse::<u32>(),
                date_part[4..6].parse::<u32>(),
                date_part[6..8].parse::<u32>(),
            ) {
                let version = semver::Version::new(year as u64, month as u64, day as u64);
                // Determine which API version to use based on the firmware date
                if semver::VersionReq::parse(">=2024.11.0")
                    .unwrap()
                    .matches(&version)
                {
                    get_model_whatsminer_v3(ip).await
                } else {
                    get_model_whatsminer_v2(ip).await
                }
            } else {
                None
            }
        }
        None => None,
    }
}

pub(crate) async fn get_version_whatsminer(ip: IpAddr) -> Option<semver::Version> {
    let response = util::send_rpc_command(&ip, "get_version").await;

    match response {
        Some(json_data) => {
            let fw_version: Option<&str> = json_data["Msg"]["fw_ver"].as_str();
            fw_version?;

            let fw_version = fw_version.unwrap();

            // Parse the firmware version format: YYYYMMDD.XX.REL
            // Extract the date components
            if fw_version.len() < 8 {
                return None;
            }

            let date_part = &fw_version[..8];
            if let (Ok(year), Ok(month), Ok(day)) = (
                date_part[..4].parse::<u32>(),
                date_part[4..6].parse::<u32>(),
                date_part[6..8].parse::<u32>(),
            ) {
                let version = semver::Version::new(year as u64, month as u64, day as u64);
                Some(version)
            } else {
                None
            }
        }
        None => None,
    }
}

pub(crate) async fn get_model_bitaxe(ip: IpAddr) -> Option<MinerModel> {
    let raw_json = util::send_web_command(&ip, "/api/system/info")
        .await
        .unwrap()
        .0;
    let response: serde_json::Value = serde_json::from_str(&raw_json).ok()?;

    let model = response["ASICModel"].as_str()?;

    MinerModelFactory::new()
        .with_make(MinerMake::Bitaxe)
        .parse_model(model)
}
pub(crate) async fn get_version_bitaxe(ip: IpAddr) -> Option<semver::Version> {
    let raw_json = util::send_web_command(&ip, "/api/system/info")
        .await
        .unwrap()
        .0;
    let response: serde_json::Value = serde_json::from_str(&raw_json).ok()?;

    match response["version"].as_str() {
        Some(v) => {
            let mut version = semver::Version::parse(v.strip_prefix("v")?).ok()?;
            version.pre = semver::Prerelease::EMPTY;
            version.build = semver::BuildMetadata::EMPTY;
            Some(version)
        }
        _ => None,
    }
}

pub(crate) async fn get_model_avalonminer(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "version").await;

    match response {
        Some(json_data) => {
            if let Some(model_field) = json_data.pointer("/VERSION/0/MODEL")
                && let Some(model_str) = model_field.as_str()
            {
                let model = model_str.split("-").collect::<Vec<&str>>()[0].to_uppercase();
                return MinerModelFactory::new()
                    .with_make(MinerMake::AvalonMiner)
                    .parse_model(&model);
            }

            None
        }
        None => None,
    }
}
pub(crate) async fn get_model_luxos(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "version").await;
    match response {
        Some(json_data) => {
            let model = json_data["VERSION"][0]["Type"].as_str();
            model?;
            let model = model.unwrap().to_uppercase();

            MinerModelFactory::new()
                .with_firmware(MinerFirmware::LuxOS)
                .parse_model(&model)
        }
        None => None,
    }
}

pub(crate) async fn get_model_braiins_os(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "devdetails").await;
    match response {
        Some(json_data) => {
            let model = json_data["DEVDETAILS"][0]["Model"].as_str();
            model?;
            let model = model
                .unwrap()
                .to_uppercase()
                .replace("BITMAIN ", "")
                .replace("S19XP", "S19 XP");

            MinerModelFactory::new()
                .with_firmware(MinerFirmware::BraiinsOS)
                .parse_model(&model)
        }
        None => None,
    }
}

pub(crate) async fn get_model_marathon(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "version").await;

    match response {
        Some(json_data) => {
            let model: Option<&str> = json_data["VERSION"][0]["Model"].as_str();
            model?;

            let model = model.unwrap().to_uppercase();

            MinerModelFactory::new()
                .with_firmware(MinerFirmware::Marathon)
                .parse_model(&model)
        }
        None => None,
    }
}
