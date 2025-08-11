use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinerMessage {
    /// The time this message was generated or occurred
    pub timestamp: u32,
    /// The message code
    /// May be set to 0 if no code is set by the device
    pub code: u64,
    /// The human-readable message being relayed by the device
    pub message: String,
    /// The severity of this message
    pub severity: MessageSeverity,
}

impl MinerMessage {
    pub fn new(timestamp: u32, code: u64, message: String, severity: MessageSeverity) -> Self {
        Self {
            timestamp,
            code,
            message,
            severity,
        }
    }
}
