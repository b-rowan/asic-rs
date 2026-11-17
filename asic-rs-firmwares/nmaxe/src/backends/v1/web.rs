use std::{net::IpAddr, time::Duration};

use once_cell::sync::OnceCell;

use asic_rs_core::{data::command::MinerCommand, traits::miner::*};
use async_trait::async_trait;
use reqwest::{Client, Method, Response};
use serde_json::Value;
use tokio::time::timeout;

#[derive(Debug)]
pub struct NMAxeWebAPI {
    client: OnceCell<Client>,
    pub ip: IpAddr,
    port: u16,
    timeout: Duration,
    retries: u32,
}

#[async_trait]
impl APIClient for NMAxeWebAPI {
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
impl WebAPIClient for NMAxeWebAPI {
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        parameters: Option<Value>,
        method: Method,
    ) -> anyhow::Result<Value> {
        let url = format!(
            "http://{}/api/{}",
            std::net::SocketAddr::new(self.ip, self.port),
            command
        );

        // Retry reads only; a repeated mutation may reboot the device twice.
        let retries = if method == Method::GET {
            self.retries
        } else {
            0
        };
        for attempt in 0..=retries {
            let result = self
                .execute_request(&url, &method, parameters.clone())
                .await;

            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        let body = response.text().await?;
                        if method != Method::GET {
                            return Ok(serde_json::from_str(&body).unwrap_or(Value::String(body)));
                        }
                        match serde_json::from_str(&body) {
                            Ok(json_data) => return Ok(json_data),
                            Err(e) => {
                                if attempt == retries {
                                    Err(NMAxeError::ParseError(e.to_string()))?;
                                }
                            }
                        }
                    } else if attempt == retries {
                        Err(NMAxeError::HttpError(response.status().as_u16()))?;
                    }
                }
                Err(e) => {
                    if attempt == retries {
                        Err(e)?;
                    }
                }
            }
        }

        Err(NMAxeError::MaxRetriesExceeded)?
    }
}

impl NMAxeWebAPI {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            client: OnceCell::new(),
            ip,
            port,
            timeout: Duration::from_secs(5),
            retries: 1,
        }
    }

    fn build_client() -> Result<Client, NMAxeError> {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| NMAxeError::RequestError(format!("failed to create HTTP client: {e}")))
    }

    fn client(&self) -> Result<&Client, NMAxeError> {
        self.client.get_or_try_init(Self::build_client)
    }

    async fn execute_request(
        &self,
        url: &str,
        method: &Method,
        parameters: Option<Value>,
    ) -> anyhow::Result<Response, NMAxeError> {
        let client = self.client()?;

        let request_builder = match *method {
            Method::GET => client.get(url),
            Method::POST => {
                let mut builder = client.post(url);
                if let Some(params) = parameters {
                    builder = builder.json(&params);
                }
                builder
            }
            Method::PATCH => {
                let mut builder = client.patch(url);
                if let Some(params) = parameters {
                    builder = builder.json(&params);
                }
                builder
            }
            _ => return Err(NMAxeError::UnsupportedMethod(method.to_string())),
        };

        let request = request_builder
            .timeout(self.timeout)
            .build()
            .map_err(|e| NMAxeError::RequestError(e.to_string()))?;

        let response = timeout(self.timeout, client.execute(request))
            .await
            .map_err(|_| NMAxeError::Timeout)?
            .map_err(|e| NMAxeError::NetworkError(e.to_string()))?;
        Ok(response)
    }
}

#[derive(Debug, Clone)]
pub enum NMAxeError {
    NetworkError(String),
    HttpError(u16),
    ParseError(String),
    RequestError(String),
    Timeout,
    UnsupportedMethod(String),
    MaxRetriesExceeded,
}

impl std::fmt::Display for NMAxeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NMAxeError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            NMAxeError::HttpError(code) => write!(f, "HTTP error: {code}"),
            NMAxeError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            NMAxeError::RequestError(msg) => write!(f, "Request error: {msg}"),
            NMAxeError::Timeout => write!(f, "Request timeout"),
            NMAxeError::UnsupportedMethod(method) => write!(f, "Unsupported method: {method}"),
            NMAxeError::MaxRetriesExceeded => write!(f, "Maximum retries exceeded"),
        }
    }
}

impl std::error::Error for NMAxeError {}
