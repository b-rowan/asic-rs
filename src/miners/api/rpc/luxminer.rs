use crate::miners::api::rpc::errors::RPCError;
use crate::miners::api::rpc::status::RPCCommandStatus;
use crate::miners::api::rpc::traits::SendRPCCommand;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct LUXMinerRPC {
    ip: IpAddr,
    port: u16,
}

impl LUXMinerRPC {
    pub fn new(ip: IpAddr, port: Option<u16>) -> Self {
        Self {
            ip,
            port: port.unwrap_or(4028),
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
impl SendRPCCommand for LUXMinerRPC {
    async fn send_command(
        &self,
        command: &'static str,
        param: Option<Value>,
    ) -> Result<Value, RPCError> {
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

    fn parse_rpc_result(&self, response: &str) -> Result<Value, RPCError> {
        let status = RPCCommandStatus::from_luxminer(response)?;

        match status.into_result() {
            Ok(_) => Ok(serde_json::from_str(response)?),
            Err(e) => Err(e),
        }
    }
}
