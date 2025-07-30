use crate::data::device::models::MinerModelFactory;
use crate::data::device::{MinerMake, MinerModel};
use crate::miners::api::rpc::{btminer::BTMinerV3RPC, traits::SendRPCCommand};
use crate::miners::util;
use serde_json::json;
use std::net::IpAddr;

pub(crate) async fn get_model_whatsminer_v2(ip: IpAddr) -> Option<MinerModel> {
    let response = util::send_rpc_command(&ip, "devdetails").await;
    match response {
        Some(json_data) => {
            let model = json_data["DEVDETAILS"][0]["Model"].as_str();
            model?;
            let mut model = model.unwrap().to_uppercase().replace("_", "");
            model.pop();
            model.push('0');

            MinerModelFactory::new()
                .with_make(MinerMake::WhatsMiner)
                .parse_model(&model)
        }
        None => None,
    }
}

pub(crate) async fn get_model_whatsminer_v3(ip: IpAddr) -> Option<MinerModel> {
    let rpc = BTMinerV3RPC::new(ip, None);
    let response = rpc
        .send_command("get.device.info", Some(json!("miner")))
        .await;

    match response {
        Ok(json_data) => {
            let model = json_data["msg"]["miner"]["type"].as_str();

            model?;

            let mut model = model.unwrap().to_uppercase().replace("_", "");
            model.pop();
            model.push('0');

            MinerModelFactory::new()
                .with_make(MinerMake::WhatsMiner)
                .parse_model(&model)
        }
        Err(_) => None,
    }
}
