use crate::miners::api::rpc::errors::RPCError;
use crate::miners::api::rpc::status::RPCCommandStatus;
use crate::miners::api::rpc::traits::SendRPCCommand;
use dyn_serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
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

impl SendRPCCommand for LUXMinerRPC {
    async fn send_command<T>(
        &self,
        command: &'static str,
        param: Option<Box<dyn Serialize>>,
    ) -> Result<T, RPCError>
    where
        T: DeserializeOwned,
    {
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

        self.parse_rpc_result::<T>(&response)
    }

    fn parse_rpc_result<T>(&self, response: &str) -> Result<T, RPCError>
    where
        T: DeserializeOwned,
    {
        let status = RPCCommandStatus::from_luxminer(response)?;

        match status.into_result() {
            Ok(_) => Ok(serde_json::from_str(response)?),
            Err(e) => Err(e),
        }
    }
}
