use crate::data::device::MinerModel;
use crate::miners::factory::model::whatsminer::{get_model_whatsminer_v2, get_model_whatsminer_v3};
use crate::miners::util;
use diqwest::WithDigestAuth;
use reqwest::{Client, Response};
use std::net::IpAddr;

pub mod whatsminer;

pub(crate) async fn get_model_antminer(ip: IpAddr) -> Option<MinerModel> {
    let response: Option<Response> = Client::new()
        .get(format!("http://{}/cgi-bin/get_system_info.cgi", ip))
        .send_with_digest_auth("root", "root")
        .await
        .ok();
    match response {
        Some(data) => {
            let json_data = data.json::<serde_json::Value>().await.ok()?;
            let model = json_data["minertype"].as_str().unwrap_or("").to_uppercase();

            Some(MinerModel::AntMiner(None).parse_model_str(&model))
        }
        None => None,
    }
}

pub(crate) async fn get_model_whatsminer(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "get_version").await;

    match response {
        Some(json_data) => {
            let fw_version: Option<&str> = json_data["Msg"]["fw_ver"].as_str();
            if fw_version.is_none() {
                return None;
            }

            let fw_version = fw_version.unwrap();

            // Parse the firmware version format: YYYYMMDD.XX.REL
            // Extract the date components
            if fw_version.len() < 8 {
                return None;
            }

            let date_part = &fw_version[..8];
            if let (Ok(year), Ok(month), Ok(_day)) = (
                date_part[..4].parse::<u32>(),
                date_part[4..6].parse::<u32>(),
                date_part[6..8].parse::<u32>(),
            ) {
                // Determine which API version to use based on the firmware date
                if year >= 2025 || (year == 2024 && month >= 11) {
                    get_model_whatsminer_v3(ip).await
                } else {
                    get_model_whatsminer_v2(ip).await
                }
            } else {
                return None;
            }
        }
        None => None,
    }
}

pub(crate) async fn get_model_luxos(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "version").await;
    match response {
        Some(json_data) => {
            let model = json_data["VERSION"][0]["Type"].as_str();
            if model.is_none() {
                return None;
            }
            let model = model.unwrap().to_uppercase();

            Some(MinerModel::AntMiner(None).parse_model_str(&model))
        }
        None => None,
    }
}

pub(crate) async fn get_model_braiins_os(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "devdetails").await;
    match response {
        Some(json_data) => {
            let model = json_data["DEVDETAILS"][0]["Model"].as_str();
            if model.is_none() {
                return None;
            }
            let model = model
                .unwrap()
                .to_uppercase()
                .replace("BITMAIN ", "")
                .replace("S19XP", "S19 XP");

            Some(MinerModel::AntMiner(None).parse_model_str(&model))
        }
        None => None,
    }
}
