use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum MinerCommand {
    RPC {
        command: &'static str,
        parameters: Option<Value>,
    },
    GRPC {
        command: &'static str,
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
