use crate::miners::api::rpc::status::RPCCommandStatus;
use crate::miners::api::rpc::traits::SendRPCCommand;
use crate::miners::api::{ApiClient, rpc::errors::RPCError};
use crate::miners::commands::MinerCommand;
use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct BTMinerV3RPC {
    ip: IpAddr,
    port: u16,
}

impl BTMinerV3RPC {
    pub fn new(ip: IpAddr, port: Option<u16>) -> Self {
        Self {
            ip,
            port: port.unwrap_or(4433),
        }
    }
}

#[async_trait]
impl ApiClient for BTMinerV3RPC {
    async fn get_api_result(&self, command: &MinerCommand) -> Result<Value, String> {
        match command {
            MinerCommand::RPC {
                command,
                parameters,
            } => self
                .send_command(command.clone(), parameters.clone())
                .await
                .map_err(|e| e.to_string()),
            _ => Result::Err("Cannot send non web command to web API".to_string()),
        }
    }
}

impl RPCCommandStatus {
    fn from_btminer_v3(response: &str) -> Result<Self, RPCError> {
        let value: serde_json::Value = serde_json::from_str(response)?;

        match value["code"].as_i64() {
            None => {
                let message = value["msg"].as_str();

                Err(RPCError::StatusCheckFailed(
                    message
                        .unwrap_or("Unknown error when looking for status code")
                        .to_owned(),
                ))
            }
            Some(code) => match code {
                0 => Ok(Self::Success),
                _ => {
                    let message = value["msg"].as_str();
                    Err(RPCError::StatusCheckFailed(
                        message
                            .unwrap_or("Unknown error when parsing status")
                            .to_owned(),
                    ))
                }
            },
        }
    }
}

#[async_trait]
impl SendRPCCommand for BTMinerV3RPC {
    async fn send_command<T, P>(
        &self,
        command: &'static str,
        parameters: Option<P>,
    ) -> Result<T, RPCError>
    where
        T: DeserializeOwned,
        P: Serialize + Send,
    {
        let mut stream = tokio::net::TcpStream::connect((self.ip, self.port))
            .await
            .map_err(|_| RPCError::ConnectionFailed)?;

        let request = match parameters {
            Some(p) => {
                json!({ "cmd": command, "param": p })
            }
            None => json!({ "cmd": command }),
        };
        let json_str = request.to_string();
        let json_bytes = json_str.as_bytes();
        let length = json_bytes.len() as u32;

        stream.write_all(&length.to_le_bytes()).await?;
        stream.write_all(&json_bytes).await?;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let response_len = u32::from_le_bytes(len_buf) as usize;

        let mut resp_buf = vec![0u8; response_len];
        stream.read_exact(&mut resp_buf).await?;

        let response_str = String::from_utf8_lossy(&resp_buf).into_owned();

        self.parse_rpc_result::<T>(&response_str)
    }

    fn parse_rpc_result<T>(&self, response: &str) -> Result<T, RPCError>
    where
        T: DeserializeOwned,
    {
        let status = RPCCommandStatus::from_btminer_v3(response)?;
        match status.into_result() {
            Ok(_) => Ok(serde_json::from_str(response)?),
            Err(e) => Err(e),
        }
    }
}
