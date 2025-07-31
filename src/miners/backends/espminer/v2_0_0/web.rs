use crate::miners::{
    api::{APIClient, WebAPIClient},
    commands::MinerCommand,
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::{Client, Method, Response};
use serde_json::Value;
use std::{net::IpAddr, time::Duration};
use tokio::time::timeout;

/// ESPMiner WebAPI client for communicating with BitAxe and similar miners
#[derive(Debug)]
pub struct ESPMinerWebAPI {
    client: Client,
    pub ip: IpAddr,
    port: u16,
    timeout: Duration,
    retries: u32,
}

#[async_trait]
#[allow(dead_code)]
trait ESPMiner200WebAPI: WebAPIClient {
    /// Get system information
    async fn system_info(&self) -> Result<Value> {
        self.send_command("system/info", false, None, Method::GET)
            .await
    }

    /// Get swarm information
    async fn swarm_info(&self) -> Result<Value> {
        self.send_command("swarm/info", false, None, Method::GET)
            .await
    }

    /// Restart the system
    async fn restart(&self) -> Result<Value> {
        self.send_command("system/restart", false, None, Method::POST)
            .await
    }

    /// Update system settings
    async fn update_settings(&self, config: Value) -> Result<Value> {
        self.send_command("system", false, Some(config), Method::PATCH)
            .await
    }
}

#[async_trait]
impl APIClient for ESPMinerWebAPI {
    async fn get_api_result(&self, command: &MinerCommand) -> Result<Value> {
        match command {
            MinerCommand::WebAPI {
                command,
                parameters,
            } => self
                .send_command(command, false, parameters.clone(), Method::GET)
                .await
                .map_err(|e| anyhow!(e.to_string())),
            _ => Err(anyhow!("Cannot send non web command to web API")),
        }
    }
}

#[async_trait]
impl WebAPIClient for ESPMinerWebAPI {
    /// Send a command to the miner
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        parameters: Option<Value>,
        method: Method,
    ) -> Result<Value> {
        let url = format!("http://{}:{}/api/{}", self.ip, self.port, command);

        for attempt in 0..=self.retries {
            let result = self
                .execute_request(&url, &method, parameters.clone())
                .await;

            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json().await {
                            Ok(json_data) => return Ok(json_data),
                            Err(e) => {
                                if attempt == self.retries {
                                    return Err(ESPMinerError::ParseError(e.to_string()))?;
                                }
                            }
                        }
                    } else if attempt == self.retries {
                        return Err(ESPMinerError::HttpError(response.status().as_u16()))?;
                    }
                }
                Err(e) => {
                    if attempt == self.retries {
                        return Err(e)?;
                    }
                }
            }
        }

        Err(ESPMinerError::MaxRetriesExceeded)?
    }
}

impl ESPMiner200WebAPI for ESPMinerWebAPI {}

impl ESPMinerWebAPI {
    /// Create a new ESPMiner WebAPI client
    pub fn new(ip: IpAddr, port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            ip,
            port,
            timeout: Duration::from_secs(5),
            retries: 1,
        }
    }

    /// Set the timeout for API requests
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the number of retries for failed requests
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Execute the actual HTTP request
    async fn execute_request(
        &self,
        url: &str,
        method: &Method,
        parameters: Option<Value>,
    ) -> Result<Response, ESPMinerError> {
        let request_builder = match *method {
            Method::GET => self.client.get(url),
            Method::POST => {
                let mut builder = self.client.post(url);
                if let Some(params) = parameters {
                    builder = builder.json(&params);
                }
                builder
            }
            Method::PATCH => {
                let mut builder = self.client.patch(url);
                if let Some(params) = parameters {
                    builder = builder.json(&params);
                }
                builder
            }
            _ => return Err(ESPMinerError::UnsupportedMethod(method.to_string())),
        };

        let request = request_builder
            .timeout(self.timeout)
            .build()
            .map_err(|e| ESPMinerError::RequestError(e.to_string()))?;

        let response = timeout(self.timeout, self.client.execute(request))
            .await
            .map_err(|_| ESPMinerError::Timeout)?
            .map_err(|e| ESPMinerError::NetworkError(e.to_string()))?;
        Ok(response)
    }
}

/// Error types for ESPMiner WebAPI operations
#[derive(Debug, Clone)]
pub enum ESPMinerError {
    /// Network error (connection issues, DNS resolution, etc.)
    NetworkError(String),
    /// HTTP error with status code
    HttpError(u16),
    /// JSON parsing error
    ParseError(String),
    /// Request building error
    RequestError(String),
    /// Timeout error
    Timeout,
    /// Unsupported HTTP method
    UnsupportedMethod(String),
    /// Maximum retries exceeded
    MaxRetriesExceeded,
    WebError,
}

impl std::fmt::Display for ESPMinerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ESPMinerError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ESPMinerError::HttpError(code) => write!(f, "HTTP error: {}", code),
            ESPMinerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ESPMinerError::RequestError(msg) => write!(f, "Request error: {}", msg),
            ESPMinerError::Timeout => write!(f, "Request timeout"),
            ESPMinerError::UnsupportedMethod(method) => write!(f, "Unsupported method: {}", method),
            ESPMinerError::MaxRetriesExceeded => write!(f, "Maximum retries exceeded"),
            ESPMinerError::WebError => write!(f, "Web error"),
        }
    }
}

impl std::error::Error for ESPMinerError {}

// Usage example
#[cfg(test)]
mod tests {
    /*
    #[tokio::test]
    async fn test_espminer_api() {
        let api = EspWebApi::new("192.168.1.100".into(), 80)
            .with_timeout(Duration::from_secs(5))
            .with_retries(3);

        // Test system info
        match api.system_info().await {
            Ok(info) => println!("System info: {:?}", info),
            Err(e) => println!("Error getting system info: {}", e),
        }
    }
     */
}
