use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;

use crate::miners::backends::traits::*;

pub use super::super::v2_0_0::web::BitaxeWebAPI;

#[async_trait]
#[allow(dead_code)]
trait Bitaxe290WebAPI: WebAPIClient {
    /// Get ASIC information
    async fn asic_info(&self) -> Result<Value> {
        self.send_command("system/asic", false, None, Method::GET)
            .await
    }
}

impl Bitaxe290WebAPI for BitaxeWebAPI {}
