use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;
use std::fmt::Debug;

use crate::miners::commands::MinerCommand;

use crate::data::miner::MinerData;
use crate::miners::data::{DataField, DataLocation};

/// Trait that every miner backend must implement to provide miner data.
#[async_trait]
pub trait GetMinerData: Send + Sync + Debug {
    /// Asynchronously retrieves standardized information about a miner,
    /// returning it as a `MinerData` struct.
    async fn get_data(&self) -> MinerData;

    /// Returns the locations of the specified data field on the miner.
    ///
    /// This associates API commands (routes) with `DataExtractor` structs,
    /// describing how to extract the data for a given `DataField`.
    fn get_locations(&self, data_field: DataField) -> Vec<DataLocation>;
}

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
