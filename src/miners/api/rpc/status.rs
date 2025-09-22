use crate::miners::api::rpc::errors::RPCError;
use serde_json::Value;

pub enum RPCCommandStatus {
    Success,
    Information,
    Error(String),
    Unknown,
}

impl RPCCommandStatus {
    pub fn into_result(self) -> Result<(), RPCError> {
        match self {
            RPCCommandStatus::Success => Ok(()),
            RPCCommandStatus::Information => Ok(()),
            RPCCommandStatus::Error(msg) => Err(RPCError::StatusCheckFailed(msg)),
            RPCCommandStatus::Unknown => {
                Err(RPCError::StatusCheckFailed(String::from("Unknown status")))
            }
        }
    }

    pub fn from_str(response: &str, message: Option<&str>) -> Self {
        match response {
            "S" => RPCCommandStatus::Success,
            "I" => RPCCommandStatus::Information,
            "E" => RPCCommandStatus::Error(message.unwrap_or("Unknown error").to_string()),
            _ => RPCCommandStatus::Unknown,
        }
    }

    pub fn from_luxminer(response: &str) -> Result<Self, RPCError> {
        let json: Value = serde_json::from_str(response)
            .map_err(|_| RPCError::StatusCheckFailed("Invalid JSON response".to_string()))?;

        let status = json
            .pointer("/STATUS/0/STATUS")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                RPCError::StatusCheckFailed(
                    "Failed to parse status from LuxMiner response".to_string(),
                )
            })?;

        let message = json.pointer("/STATUS/0/Msg").and_then(|v| v.as_str());

        Ok(Self::from_str(status, message))
    }
}
