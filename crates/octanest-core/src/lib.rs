//! Shared domain types for Octanest.

pub mod auth_types;
pub mod pat_types;
pub mod repo_types;

pub use auth_types::*;
pub use pat_types::*;
pub use repo_types::*;

use serde::{Deserialize, Serialize};

pub fn crate_name() -> &'static str {
    "octanest-core"
}

pub const RPC_PROTOCOL_VERSION: u32 = 1;
pub const ECHO_MAX_BYTES: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    /// `"ok"`, `"error"`, or `"skipped"` when no DATABASE_URL / ping not attempted.
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbProbeResponse {
    pub dialect: String,
    pub probe_count: i64,
    pub probed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcInput {
    Empty(serde_json::Value),
    Echo(EchoRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub procedure: String,
    #[serde(default = "default_input")]
    pub input: serde_json::Value,
}

fn default_input() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcResponse {
    Ok { ok: bool, data: serde_json::Value },
    Err { ok: bool, error: AppError },
}

impl RpcResponse {
    pub fn ok(data: impl Serialize) -> Self {
        Self::Ok {
            ok: true,
            data: serde_json::to_value(data).unwrap_or(serde_json::json!({})),
        }
    }

    pub fn err(error: AppError) -> Self {
        Self::Err { ok: false, error }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert_eq!(crate_name(), "octanest-core");
    }
}
