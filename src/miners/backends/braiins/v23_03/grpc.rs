pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/1.0.0-alpha/braiins.bos.v1.rs"));
    pub static DESCRIPTORS: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/1.0.0-alpha/descriptor.bin"));
}

use crate::miners::backends::traits::APIClient;
use crate::miners::backends::traits::GRPCAPIClient;
use crate::miners::commands::MinerCommand;
use async_trait::async_trait;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor};
use proto::{LoginRequest, authentication_service_client::AuthenticationServiceClient};
use serde_json::{Deserializer, Value};
use std::net::IpAddr;
use tokio::sync::RwLock;
use tonic::Request;
use tonic_prost::ProstCodec;

fn descriptor_pool() -> DescriptorPool {
    DescriptorPool::decode(proto::DESCRIPTORS).unwrap()
}

pub struct BraiinsGRPCAPI {
    pub ip: IpAddr,
    port: u16,
    auth_token: RwLock<Option<String>>,
    username: Option<String>,
    password: Option<String>,
}

#[async_trait]
impl APIClient for BraiinsGRPCAPI {
    async fn get_api_result(&self, command: &MinerCommand) -> anyhow::Result<Value> {
        match command {
            MinerCommand::GRPC {
                service,
                command,
                request,
            } => self
                .send_command(service, command, false, request.clone())
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string())),
            _ => Err(anyhow::anyhow!("Cannot send non web command to web API")),
        }
    }
}

#[async_trait]
impl GRPCAPIClient for BraiinsGRPCAPI {
    /// Send a command to the Braiins miner API
    async fn send_command(
        &self,
        service: &str,
        command: &str,
        _privileged: bool,
        parameters: Option<Value>,
    ) -> anyhow::Result<Value> {
        // Ensure we're authenticated before making the request
        if let Err(e) = self.ensure_authenticated().await {
            return Err(anyhow::anyhow!("Failed to authenticate: {}", e));
        }
        self.execute_request(service, command, parameters);
        Ok(Value::Null)
    }
}

impl BraiinsGRPCAPI {
    /// Create a new Braiins WebAPI client
    pub fn new(ip: IpAddr) -> Self {
        Self {
            ip,
            port: 50051,
            auth_token: RwLock::new(None),
            username: Some("root".to_string()), // Default user
            password: Some("root".to_string()), // Default password
        }
    }

    /// Ensure authentication token is present, authenticate if needed
    pub async fn ensure_authenticated(&self) -> anyhow::Result<()> {
        let mut client = AuthenticationServiceClient::connect(format!(
            "http://{}:{}",
            self.ip.to_string(),
            self.port
        ))
        .await?;

        let request = tonic::Request::new(LoginRequest {
            username: self.username.clone().unwrap_or("".to_string()),
            password: self.password.clone().unwrap_or("".to_string()),
        });

        let response = client.login(request).await?;

        let auth_token = response.metadata().get("authorization").clone();

        match auth_token {
            None => {
                anyhow::bail!("Failed to get authorization token");
            }
            Some(token) => {
                self.auth_token
                    .write()
                    .await
                    .replace(token.to_str()?.to_string());
            }
        }
        Ok(())
    }

    pub async fn execute_request(
        &self,
        service: &str,
        request: &str,
        parameters: Option<Value>,
    ) -> anyhow::Result<()> {
        let pool = descriptor_pool();

        let message_name = format!("{service}{request}");
        let request = Self::json_to_prost(&pool, &message_name, parameters)?;

        Ok(())
    }

    async fn call_grpc(
        &self,
        command: &str,
        request: DynamicMessage,
    ) -> anyhow::Result<DynamicMessage> {
        let path = format!("/{command}");

        let transport =
            tonic::transport::Endpoint::new(format!("http://{}:{}", self.ip, self.port))?
                .connect()
                .await?;
        let mut client = tonic::client::Grpc::new(transport);

        // let response = client
        //     .unary(
        //         Request::new(request),
        //         path.parse()?,
        //         ProstCodec::new(),
        //     )
        //     .await?;
        //
        // Ok(response.into_inner())
        anyhow::bail!("Failed to send request")
    }

    fn json_to_prost(
        pool: &DescriptorPool,
        full_message_name: &str,
        json: Option<Value>,
    ) -> anyhow::Result<DynamicMessage> {
        let message_desc: MessageDescriptor = pool
            .get_message_by_name(full_message_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown message: {full_message_name}"))?;

        let msg;
        match json {
            Some(inner) => {
                let request = inner.clone().to_string();
                let mut deserializer = Deserializer::from_str(request.as_str());
                msg = DynamicMessage::deserialize(message_desc.clone(), &mut deserializer)?;
            }
            None => {
                msg = DynamicMessage::new(message_desc.clone());
            }
        }
        Ok(msg)
    }
}
