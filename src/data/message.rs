#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinerMessage {
    pub timestamp: u32,
    pub code: u64,
    pub message: String,
    pub severity: MessageSeverity,
}
