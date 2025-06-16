mod commands;
mod hardware;
mod model;
mod traits;

use futures::future::FutureExt;
use futures::pin_mut;
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use std::net::IpAddr;
use std::time::Duration;
use std::{collections::HashSet, error::Error};
use tokio::task::JoinSet;

use super::commands::MinerCommand;
use super::util::{send_rpc_command, send_web_command};
use crate::data::device::{MinerFirmware, MinerMake, MinerModel};
use crate::miners::backends::btminer::BTMinerV3Backend;
use crate::miners::backends::traits::GetMinerData;
use traits::{DiscoveryCommands, ModelSelection};

const MAX_WAIT_TIME: Duration = Duration::from_secs(5);

async fn get_miner_type_from_command(
    ip: IpAddr,
    command: MinerCommand,
) -> Option<(Option<MinerModel>, Option<MinerFirmware>)> {
    match command {
        MinerCommand::RPC { command } => {
            let response = send_rpc_command(&ip, command).await?;
            parse_type_from_socket(response)
        }
        MinerCommand::WebAPI { command } => {
            let response = send_web_command(&ip, command).await?;
            parse_type_from_web(response)
        }
        _ => None,
    }
}

fn parse_type_from_socket(
    response: serde_json::Value,
) -> Option<(Option<MinerModel>, Option<MinerFirmware>)> {
    let json_string = response.to_string().to_uppercase();

    match () {
        _ if json_string.contains("BOSMINER") || json_string.contains("BOSER") => {
            Some((None, Some(MinerFirmware::BraiinsOS)))
        }
        _ if json_string.contains("LUXMINER") => Some((None, Some(MinerFirmware::LuxOS))),
        _ if json_string.contains("BITMICRO") || json_string.contains("BTMINER") => Some((
            Some(MinerModel::WhatsMiner(None)),
            Some(MinerFirmware::Stock),
        )),
        _ if json_string.contains("ANTMINER") && !json_string.contains("DEVDETAILS") => {
            Some((Some(MinerModel::AntMiner(None)), Some(MinerFirmware::Stock)))
        }
        _ if json_string.contains("AVALON") => Some((
            Some(MinerModel::AvalonMiner(None)),
            Some(MinerFirmware::Stock),
        )),
        _ if json_string.contains("VNISH") => Some((None, Some(MinerFirmware::VNish))),
        _ => None,
    }
}

fn parse_type_from_web(
    response: (String, HeaderMap, StatusCode),
) -> Option<(Option<MinerModel>, Option<MinerFirmware>)> {
    let (resp_text, resp_headers, resp_status) = response;

    let auth_header = match resp_headers.get("www-authenticate") {
        Some(header) => header.to_str().unwrap(),
        None => "",
    };
    let redirect_header = match resp_headers.get("location") {
        Some(header) => header.to_str().unwrap(),
        None => "",
    };

    match () {
        _ if resp_status == 401 && auth_header.contains("realm=\"antMiner") => {
            Some((Some(MinerModel::AntMiner(None)), Some(MinerFirmware::Stock)))
        }
        _ if resp_text.contains("Braiins OS") => Some((None, Some(MinerFirmware::BraiinsOS))),
        _ if resp_text.contains("Luxor Firmware") => Some((None, Some(MinerFirmware::LuxOS))),
        _ if resp_text.contains("AxeOS") => {
            Some((Some(MinerModel::BitAxe(None)), Some(MinerFirmware::Stock)))
        }
        _ if resp_text.contains("Miner Web Dashboard") => Some((None, Some(MinerFirmware::EPic))),
        _ if resp_text.contains("Avalon") => Some((
            Some(MinerModel::AvalonMiner(None)),
            Some(MinerFirmware::Stock),
        )),
        _ if resp_text.contains("AnthillOS") => Some((None, Some(MinerFirmware::VNish))),
        _ if redirect_header.contains("https://") && resp_status == 307
            || resp_text.contains("/cgi-bin/luci") =>
        {
            Some((
                Some(MinerModel::WhatsMiner(None)),
                Some(MinerFirmware::Stock),
            ))
        }
        _ => None,
    }
}
fn select_backend(
    ip: IpAddr,
    model: Option<MinerModel>,
    firmware: Option<MinerFirmware>,
) -> Option<Box<impl GetMinerData>> {
    match (model, firmware) {
        (Some(MinerModel::WhatsMiner(_)), Some(MinerFirmware::Stock)) => Some(Box::new(
            BTMinerV3Backend::new(ip, model.expect("Could not find model")),
        )),
        _ => None,
    }
}

pub struct MinerFactory {
    search_makes: Option<Vec<MinerMake>>,
    search_firmwares: Option<Vec<MinerFirmware>>,
}
impl MinerFactory {
    pub async fn get_miner(
        self,
        ip: IpAddr,
    ) -> Result<Option<Box<impl GetMinerData>>, Box<dyn Error>> {
        let search_makes = self.search_makes.clone().unwrap_or(vec![
            MinerMake::AntMiner,
            MinerMake::WhatsMiner,
            MinerMake::AvalonMiner,
            MinerMake::EPic,
            MinerMake::Braiins,
            MinerMake::BitAxe,
        ]);
        let search_firmwares = self.search_firmwares.clone().unwrap_or(vec![
            MinerFirmware::Stock,
            MinerFirmware::BraiinsOS,
            MinerFirmware::VNish,
            MinerFirmware::EPic,
            MinerFirmware::HiveOS,
            MinerFirmware::LuxOS,
            MinerFirmware::Marathon,
            MinerFirmware::MSKMiner,
        ]);
        let mut commands: HashSet<MinerCommand> = HashSet::new();

        for make in search_makes {
            for command in make.get_discovery_commands() {
                commands.insert(command);
            }
        }
        for firmware in search_firmwares {
            for command in firmware.get_discovery_commands() {
                commands.insert(command);
            }
        }

        let mut discovery_tasks = JoinSet::new();
        for command in commands {
            let _ = discovery_tasks.spawn(get_miner_type_from_command(ip, command));
        }

        let timeout = tokio::time::sleep(MAX_WAIT_TIME).fuse();
        let tasks = tokio::spawn(async move {
            loop {
                if discovery_tasks.is_empty() {
                    return None;
                };
                match discovery_tasks.join_next().await.unwrap_or(Ok(None)) {
                    Ok(Some(result)) => {
                        return Some(result);
                    }
                    _ => continue,
                };
            }
        });

        pin_mut!(timeout, tasks);

        let miner_info = tokio::select!(
            Ok(miner_info) = &mut tasks => {
                miner_info
            },
            _ = &mut timeout => {
                None
            }
        );

        match miner_info {
            Some((model, firmware)) => {
                let model = if let Some(miner_model) = model {
                    miner_model.get_model(ip).await
                } else if let Some(miner_firmware) = firmware {
                    miner_firmware.get_model(ip).await
                } else {
                    return Ok(None);
                };
                Ok(select_backend(ip, model, firmware))
            }
            None => Ok(None),
        }
    }

    pub fn new() -> MinerFactory {
        MinerFactory {
            search_makes: None,
            search_firmwares: None,
        }
    }

    pub fn with_search_makes(&mut self, search_makes: Vec<MinerMake>) -> &Self {
        self.search_makes = Some(search_makes);
        self
    }
    pub fn with_search_firmwares(&mut self, search_firmwares: Vec<MinerFirmware>) -> &Self {
        self.search_firmwares = Some(search_firmwares);
        self
    }
    pub fn add_search_make(&mut self, search_make: MinerMake) -> &Self {
        if self.search_makes.is_none() {
            self.search_makes = Some(vec![search_make]);
        }
        self.search_makes.as_mut().unwrap().push(search_make);
        self
    }
    pub fn add_search_firmware(&mut self, search_firmware: MinerFirmware) -> &Self {
        if self.search_firmwares.is_none() {
            self.search_firmwares = Some(vec![search_firmware]);
        }
        self.search_firmwares
            .as_mut()
            .unwrap()
            .push(search_firmware);
        self
    }
    pub fn remove_search_make(&mut self, search_make: MinerMake) -> &Self {
        if self.search_makes.is_none() {
            return self;
        }
        self.search_makes
            .as_mut()
            .unwrap()
            .retain(|val| *val != search_make);
        self
    }
    pub fn remove_search_firmware(&mut self, search_firmware: MinerFirmware) -> &Self {
        if self.search_firmwares.is_none() {
            return self;
        }
        self.search_firmwares
            .as_mut()
            .unwrap()
            .retain(|val| *val != search_firmware);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_from_socket_whatsminer_2024_09_30() {
        const RAW_DATA: &str = r#"{"STATUS": [{"STATUS": "S", "Msg": "Device Details"}], "DEVDETAILS": [{"DEVDETAILS": 0, "Name": "SM", "ID": 0, "Driver": "bitmicro", "Kernel": "", "Model": "M30S+_VE40"}, {"DEVDETAILS": 1, "Name": "SM", "ID": 1, "Driver": "bitmicro", "Kernel": "", "Model": "M30S+_VE40"}, {"DEVDETAILS": 2, "Name": "SM", "ID": 2, "Driver": "bitmicro", "Kernel": "", "Model": "M30S+_VE40"}], "id": 1}"#;
        let parsed_data = serde_json::from_str(RAW_DATA).unwrap();
        let result = parse_type_from_socket(parsed_data);
        assert_eq!(
            result,
            Some((
                Some(MinerModel::WhatsMiner(None)),
                Some(MinerFirmware::Stock)
            ))
        )
    }
    #[test]
    fn test_parse_type_from_web_whatsminer_2024_09_30() {
        let mut headers = HeaderMap::new();
        headers.insert("location", "https://example.com/".parse().unwrap());

        let response_data = (String::from(""), headers, StatusCode::TEMPORARY_REDIRECT);

        let result = parse_type_from_web(response_data);
        assert_eq!(
            result,
            Some((
                Some(MinerModel::WhatsMiner(None)),
                Some(MinerFirmware::Stock)
            ))
        )
    }
}
