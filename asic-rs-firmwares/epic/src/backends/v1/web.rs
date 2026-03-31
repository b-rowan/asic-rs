use std::{net::IpAddr, time::Duration};

use anyhow;
use asic_rs_core::{data::command::MinerCommand, traits::miner::*};
use async_trait::async_trait;
use reqwest::{Client, Method};
use serde_json::{Value, json};

/// ePIC PowerPlay WebAPI client
#[derive(Debug)]
pub struct PowerPlayWebAPI {
    client: Client,
    pub ip: IpAddr,
    port: u16,
    timeout: Duration,
    password: Option<String>,
}

#[async_trait]
impl APIClient for PowerPlayWebAPI {
    async fn get_api_result(&self, command: &MinerCommand) -> anyhow::Result<Value> {
        match command {
            MinerCommand::WebAPI {
                command,
                parameters,
            } => self
                .send_command(command, false, parameters.clone(), Method::GET)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string())),
            _ => Err(anyhow::anyhow!("Cannot send non web command to web API")),
        }
    }
}

#[async_trait]
impl WebAPIClient for PowerPlayWebAPI {
    /// Send a command to the EPic miner API
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        parameters: Option<Value>,
        method: Method,
    ) -> anyhow::Result<Value> {
        let url = format!("http://{}:{}/{}", self.ip, self.port, command);

        let request_builder = match method {
            Method::GET => self.client.get(&url),
            Method::POST => self.client.post(&url).json(&{
                let mut p = parameters.unwrap_or_else(|| json!({}));
                p.as_object_mut().map(|m| {
                    m.insert(
                        "password".into(),
                        Value::String(self.password.clone().unwrap_or_else(|| "letmein".into())),
                    )
                });
                p
            }),
            _ => return Err(PowerPlayError::UnsupportedMethod(method.to_string()))?,
        };

        let request = request_builder
            .timeout(self.timeout)
            .build()
            .map_err(|e| PowerPlayError::RequestError(e.to_string()))?;

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| PowerPlayError::NetworkError(e.to_string()))?;

        let status = response.status();
        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| PowerPlayError::ParseError(e.to_string()).into())
        } else {
            Err(PowerPlayError::HttpError(status.as_u16()))?
        }
    }
}

impl PowerPlayWebAPI {
    /// Create a new EPic WebAPI client
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
            password: Some("letmein".to_string()), // Default password
        }
    }
}

/// Error types for EPic WebAPI operations
#[derive(Debug, Clone)]
pub enum PowerPlayError {
    NetworkError(String),
    HttpError(u16),
    ParseError(String),
    RequestError(String),
    UnsupportedMethod(String),
}

impl std::fmt::Display for PowerPlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerPlayError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            PowerPlayError::HttpError(code) => write!(f, "HTTP error: {code}"),
            PowerPlayError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            PowerPlayError::RequestError(msg) => write!(f, "Request error: {msg}"),
            PowerPlayError::UnsupportedMethod(method) => write!(f, "Unsupported method: {method}"),
        }
    }
}

impl std::error::Error for PowerPlayError {}
