use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinerCommand {
    RPC {
        command: &'static str,
        parameters: Option<Value>,
    },
    GRPC {
        service: &'static str,
        command: &'static str,
        request: Option<Value>,
    },
    WebAPI {
        command: &'static str,
        parameters: Option<Value>,
    },
    GraphQL {
        command: &'static str,
    },
    SSH {
        command: &'static str,
    },
}
