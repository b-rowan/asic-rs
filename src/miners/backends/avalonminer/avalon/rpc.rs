use crate::miners::api::rpc::errors::RPCError;
use crate::miners::api::rpc::status::RPCCommandStatus;
use crate::miners::api::{APIClient, RPCAPIClient};
use crate::miners::commands::MinerCommand;
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub struct CGMinerRPC {
    ip: IpAddr,
    port: u16,
}

impl CGMinerRPC {
    pub fn new(ip: IpAddr) -> Self {
        Self { ip, port: 4028 }
    }

    fn parse_rpc_result(&self, response: &str) -> Result<Value> {
        let parsed: Value = serde_json::from_str(response)?;

        if let Some(status_array) = parsed.get("STATUS").and_then(|s| s.as_array()) {
            if !status_array.is_empty() {
                if let Some(status) = status_array[0].get("STATUS").and_then(|s| s.as_str()) {
                    let message = status_array[0].get("Msg").and_then(|m| m.as_str());
                    let status = RPCCommandStatus::from_str(status, message);

                    return match status.into_result() {
                        Ok(_) => Ok(parsed),
                        Err(e) => {
                            dbg!("{}: API Command Error: {}", self.ip, &e);
                            Err(anyhow!(e))
                        }
                    };
                }
            }
        }
        bail!("Invalid response format");
    }
}

#[async_trait]
impl RPCAPIClient for CGMinerRPC {
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        param: Option<Value>,
    ) -> Result<Value> {
        let cmd = match param {
            Some(params) => json!({
                "command": command,
                "parameter": params
            }),
            None => json!({
                "command": command
            }),
        };

        let stream = tokio::net::TcpStream::connect(format!("{}:{}", self.ip, self.port))
            .await
            .map_err(|_| RPCError::ConnectionFailed)?;
        let mut stream = stream;

        let json_str = cmd.to_string();
        stream.write_all(json_str.as_bytes()).await?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        if buffer.is_empty() {
            bail!("No data received from miner");
        }

        let response = String::from_utf8_lossy(&buffer)
            .into_owned()
            .replace('\0', "");

        if response == "Socket connect failed: Connection refused\n" {
            bail!("Miner connection refused");
        }

        self.parse_rpc_result(&response)
    }
}

#[async_trait]
impl APIClient for CGMinerRPC {
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
