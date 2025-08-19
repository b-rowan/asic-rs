use crate::miners::api::WebAPIClient;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;

pub use super::super::v2_0_0::web::BitAxeWebAPI;

#[async_trait]
#[allow(dead_code)]
trait BitAxe290WebAPI: WebAPIClient {
    /// Get ASIC information
    async fn asic_info(&self) -> Result<Value> {
        self.send_command("system/asic", false, None, Method::GET)
            .await
    }
}

impl BitAxe290WebAPI for BitAxeWebAPI {}
