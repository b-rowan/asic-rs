use std::{net::IpAddr, time::Duration};

use anyhow::Context;
use asic_rs_core::{
    data::{command::MinerCommand, firmware::FirmwareImage},
    traits::miner::{APIClient, ExposeSecret, MinerAuth, WebAPIClient},
};
use async_trait::async_trait;
use reqwest::{Client, Method, multipart};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
struct ProtoAuthTokens {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug)]
pub struct ProtoWebAPI {
    client: Client,
    ip: IpAddr,
    port: u16,
    auth: MinerAuth,
    tokens: Mutex<Option<ProtoAuthTokens>>,
    timeout: Duration,
}

impl ProtoWebAPI {
    pub fn new(ip: IpAddr, auth: MinerAuth) -> Self {
        Self {
            client: Client::new(),
            ip,
            port: 80,
            auth,
            tokens: Mutex::new(None),
            timeout: Duration::from_secs(10),
        }
    }

    pub fn set_auth(&mut self, auth: MinerAuth) {
        self.auth = auth;
        self.tokens = Mutex::new(None);
    }

    pub fn password(&self) -> &str {
        self.auth.password.expose_secret()
    }

    fn url(&self, command: &str) -> String {
        if command.starts_with('/') {
            format!("http://{}:{}{}", self.ip, self.port, command)
        } else {
            format!("http://{}:{}/api/v1/{}", self.ip, self.port, command)
        }
    }

    async fn access_token(&self) -> anyhow::Result<Option<String>> {
        if self.auth.password.expose_secret().is_empty() {
            return Ok(None);
        }

        if let Some(tokens) = self.tokens.lock().await.clone() {
            return Ok(Some(tokens.access_token));
        }

        let response = self
            .client
            .post(self.url("/api/v1/auth/login"))
            .timeout(self.timeout)
            .json(&json!({ "password": self.auth.password.expose_secret() }))
            .send()
            .await
            .context("Proto MDK login request failed")?;

        if !response.status().is_success() {
            anyhow::bail!("Proto MDK login failed with status {}", response.status());
        }

        let tokens = response
            .json::<ProtoAuthTokens>()
            .await
            .context("Proto MDK login response was not valid auth token JSON")?;
        let access_token = tokens.access_token.clone();
        *self.tokens.lock().await = Some(tokens);
        Ok(Some(access_token))
    }

    async fn refresh_access_token(&self) -> anyhow::Result<Option<String>> {
        let Some(refresh_token) = self
            .tokens
            .lock()
            .await
            .as_ref()
            .map(|tokens| tokens.refresh_token.clone())
        else {
            return self.access_token().await;
        };

        let response = self
            .client
            .post(self.url("/api/v1/auth/refresh"))
            .timeout(self.timeout)
            .json(&json!({ "refresh_token": refresh_token }))
            .send()
            .await
            .context("Proto MDK token refresh request failed")?;

        if !response.status().is_success() {
            *self.tokens.lock().await = None;
            return self.access_token().await;
        }

        let value = response
            .json::<Value>()
            .await
            .context("Proto MDK token refresh response was not JSON")?;
        let Some(access_token) = value.get("access_token").and_then(Value::as_str) else {
            anyhow::bail!("Proto MDK token refresh response did not include access_token");
        };

        if let Some(tokens) = self.tokens.lock().await.as_mut() {
            tokens.access_token = access_token.to_string();
        }
        Ok(Some(access_token.to_string()))
    }

    async fn request(
        &self,
        command: &str,
        privileged: bool,
        parameters: Option<Value>,
        method: Method,
    ) -> anyhow::Result<reqwest::Response> {
        let url = self.url(command);
        let mut token = if privileged {
            self.access_token().await?
        } else {
            None
        };

        for attempt in 0..=1 {
            let mut builder = match method.clone() {
                Method::GET => self.client.get(url.clone()),
                Method::POST => self.client.post(url.clone()),
                Method::PUT => self.client.put(url.clone()),
                Method::DELETE => self.client.delete(url.clone()),
                other => anyhow::bail!("Unsupported Proto MDK HTTP method {other}"),
            }
            .timeout(self.timeout);

            if let Some(parameters) = parameters.clone() {
                builder = builder.json(&parameters);
            }

            if let Some(token) = token.as_ref() {
                builder = builder.bearer_auth(token);
            }

            let response = builder
                .send()
                .await
                .with_context(|| format!("Proto MDK request to {command} failed"))?;

            if privileged && response.status() == reqwest::StatusCode::UNAUTHORIZED && attempt == 0
            {
                token = self.refresh_access_token().await?;
                continue;
            }

            return Ok(response);
        }

        anyhow::bail!("Proto MDK request retry loop exited unexpectedly")
    }

    pub async fn read_logs(&self) -> anyhow::Result<String> {
        let value = self
            .send_command("/api/v1/system/logs", false, None, Method::GET)
            .await?;
        let Some(logs) = value.get("logs") else {
            return Ok(value.to_string());
        };

        let source = logs
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("system");
        let content = logs
            .get("content")
            .and_then(Value::as_array)
            .map(|lines| {
                lines
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        Ok(format!("== {source} ==\n{content}"))
    }

    pub async fn upgrade_firmware(&self, image: FirmwareImage) -> anyhow::Result<bool> {
        let FirmwareImage { filename, bytes } = image;
        let mut builder = self
            .client
            .post(self.url("/api/v1/system/update"))
            .timeout(Duration::from_secs(300));

        if let Some(token) = self.access_token().await? {
            builder = builder.bearer_auth(token);
        }

        let form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(bytes)
                .file_name(filename)
                .mime_str("application/octet-stream")
                .context("failed to set Proto MDK firmware upload mime type")?,
        );

        let response = builder
            .multipart(form)
            .send()
            .await
            .context("Proto MDK firmware upload request failed")?;

        Ok(response.status().is_success())
    }
}

#[async_trait]
impl APIClient for ProtoWebAPI {
    async fn get_api_result(&self, command: &MinerCommand) -> anyhow::Result<Value> {
        match command {
            MinerCommand::WebAPI {
                command,
                parameters,
            } => {
                self.send_command(command, false, parameters.clone(), Method::GET)
                    .await
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported command type for Proto MDK API"
            )),
        }
    }
}

#[async_trait]
impl WebAPIClient for ProtoWebAPI {
    async fn send_command(
        &self,
        command: &str,
        privileged: bool,
        parameters: Option<Value>,
        method: Method,
    ) -> anyhow::Result<Value> {
        let response = self
            .request(command, privileged, parameters, method)
            .await?;
        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read Proto MDK response body")?;

        if !status.is_success() {
            anyhow::bail!("Proto MDK request failed with status {status}: {body}");
        }

        if body.trim().is_empty() {
            return Ok(json!({}));
        }

        serde_json::from_str(&body)
            .with_context(|| format!("Proto MDK response body was not JSON: {body}"))
    }
}
