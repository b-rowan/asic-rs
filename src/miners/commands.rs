#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MinerCommand {
    RPC { command: &'static str },
    GRPC { command: &'static str },
    WebAPI { command: &'static str },
    GraphQL { command: &'static str },
    SSH { command: &'static str },
}
