use crate::data::device::models::{MinerModelFactory, ModelSelectionError};
use crate::data::device::{MinerMake, MinerModel};
use crate::miners::backends::traits::APIClient;
use crate::miners::backends::whatsminer::v3;
use crate::miners::commands::MinerCommand;
use crate::miners::util;
use serde_json::json;
use std::net::IpAddr;
use std::result::Result as StdResult;

pub(crate) async fn get_model_whatsminer_v2(
    ip: IpAddr,
) -> StdResult<MinerModel, ModelSelectionError> {
    let response = util::send_rpc_command(&ip, "devdetails").await;
    match response {
        Some(json_data) => {
            let model = json_data["DEVDETAILS"][0]["Model"].as_str();

            if model.is_none() {
                return Err(ModelSelectionError::UnexpectedModelResponse);
            }

            let mut model = model.unwrap().to_uppercase().replace("_", "");
            model.pop();
            model.push('0');

            MinerModelFactory::new()
                .with_make(MinerMake::WhatsMiner)
                .parse_model(&model)
        }
        None => Err(ModelSelectionError::NoModelResponse),
    }
}

pub(crate) async fn get_model_whatsminer_v3(
    ip: IpAddr,
) -> StdResult<MinerModel, ModelSelectionError> {
    let rpc = v3::WhatsMinerRPCAPI::new(ip, None);
    let response = rpc
        .get_api_result(&MinerCommand::RPC {
            command: "get.device.info",
            parameters: Some(json!("miner")),
        })
        .await;

    match response {
        Ok(json_data) => {
            let model = json_data["msg"]["miner"]["type"].as_str();

            if model.is_none() {
                return Err(ModelSelectionError::UnexpectedModelResponse);
            }

            let mut model = model.unwrap().to_uppercase().replace("_", "");
            model.pop();
            model.push('0');

            MinerModelFactory::new()
                .with_make(MinerMake::WhatsMiner)
                .parse_model(&model)
        }
        Err(_) => Err(ModelSelectionError::NoModelResponse),
    }
}
