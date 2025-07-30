use async_trait::async_trait;
use serde_json::Value;

use crate::miners::commands::MinerCommand;

pub mod rpc;
pub mod web;

#[async_trait]
pub trait ApiClient: Send + Sync {
    async fn get_api_result(&self, command: &MinerCommand) -> Result<Value, String>;
}
