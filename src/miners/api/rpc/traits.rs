use crate::miners::api::rpc::errors::RPCError;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait SendRPCCommand {
    async fn send_command(
        &self,
        command: &'static str,
        parameters: Option<Value>,
    ) -> Result<Value, RPCError>;

    fn parse_rpc_result(&self, response: &str) -> Result<Value, RPCError>;
}
