use crate::data::device::MinerModel;
use crate::miners::api::rpc::{btminer::BTMinerV3RPC, traits::SendRPCCommand};
use crate::miners::util;
use serde_json::Value;
use std::net::IpAddr;

pub(crate) async fn get_model_whatsminer_v2(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "devdetails").await;
    match response {
        Some(json_data) => {
            let model = json_data["DEVDETAILS"][0]["Model"].as_str();
            if model.is_none() {
                return None;
            }
            let mut model = model.unwrap().to_uppercase().replace("_", "");
            model.pop();
            model.push('0');

            Some(MinerModel::WhatsMiner(None).parse_model_str(&model))
        }
        None => None,
    }
}

pub(crate) async fn get_model_whatsminer_v3(ip: IpAddr) -> Option<MinerModel> {
    let rpc = BTMinerV3RPC::new(ip, None);
    let response = rpc
        .send_command::<Value, &str>("get.device.info", Some("miner"))
        .await;

    match response {
        Ok(json_data) => {
            let model = json_data["msg"]["miner"]["type"].as_str();

            if model.is_none() {
                return None;
            }

            let mut model = model.unwrap().to_uppercase().replace("_", "");
            model.pop();
            model.push('0');

            Some(MinerModel::WhatsMiner(None).parse_model_str(&model))
        }
        Err(_) => None,
    }
}
