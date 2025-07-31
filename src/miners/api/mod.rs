use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;

use crate::miners::commands::MinerCommand;

pub mod rpc;

#[async_trait]
pub trait APIClient: Send + Sync {
    async fn get_api_result(&self, command: &MinerCommand) -> Result<Value>;
}

#[async_trait]
pub trait WebAPIClient: Send + Sync + APIClient {
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        parameters: Option<Value>,
        method: Method,
    ) -> Result<Value>;
}

#[async_trait]
pub trait RPCAPIClient: Send + Sync + APIClient {
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        parameters: Option<Value>,
    ) -> Result<Value>;
}
