use crate::miners::api::rpc::errors::RPCError;
use crate::miners::api::rpc::status::RPCCommandStatus;
use crate::miners::backends::traits::{APIClient, RPCAPIClient};
use crate::miners::commands::MinerCommand;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub struct LUXMinerRPCAPI {
    ip: IpAddr,
    port: u16,
}

#[async_trait]
impl APIClient for LUXMinerRPCAPI {
    async fn get_api_result(&self, command: &MinerCommand) -> Result<Value> {
        match command {
            MinerCommand::RPC {
                command,
                parameters,
            } => self
                .send_command(command, false, parameters.clone())
                .await
                .map_err(|e| anyhow!(e.to_string())),
            _ => Err(anyhow!("Cannot send non RPC command to RPC API")),
        }
    }
}

impl RPCCommandStatus {
    fn from_luxminer(response: &str) -> Result<Self, RPCError> {
        let value: serde_json::Value = serde_json::from_str(response)?;
        let message = value["STATUS"][0]["Msg"].as_str();

        match value["STATUS"][0]["STATUS"].as_str() {
            None => Err(RPCError::StatusCheckFailed(
                message
                    .unwrap_or("Unknown error when looking for status code")
                    .to_owned(),
            )),
            Some(value) => Ok(Self::from_str(value, message)),
        }
    }
}

#[async_trait]
impl RPCAPIClient for LUXMinerRPCAPI {
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        param: Option<Value>,
    ) -> Result<Value> {
        let mut stream = tokio::net::TcpStream::connect((self.ip, self.port))
            .await
            .map_err(|_| RPCError::ConnectionFailed)?;

        let request = json!({ "cmd": command, "param": param });

        stream
            .write_all(request.to_string().as_bytes())
            .await
            .unwrap();

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await.unwrap();

        let response = String::from_utf8_lossy(&buffer)
            .into_owned()
            .replace('\0', "");

        self.parse_rpc_result(&response)
    }
}

impl LUXMinerRPCAPI {
    pub fn new(ip: IpAddr, port: Option<u16>) -> Self {
        Self {
            ip,
            port: port.unwrap_or(4028),
        }
    }

    fn parse_rpc_result(&self, response: &str) -> Result<Value> {
        let status = RPCCommandStatus::from_luxminer(response)?;
        match status.into_result() {
            Ok(_) => Ok(serde_json::from_str(response)?),
            Err(e) => Err(e)?,
        }
    }
}
