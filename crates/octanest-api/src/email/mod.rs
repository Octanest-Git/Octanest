//! Outbound email adapters (log sink, SMTP, Resend).

mod log_sink;

pub use log_sink::LogSink;

use std::sync::Arc;

use thiserror::Error;

/// Outbound message payload shared by all adapters.
#[derive(Debug, Clone)]
pub struct OutboundEmail {
    pub to: String,
    pub subject: String,
    pub text: String,
    pub html: Option<String>,
}

/// Errors from email construction or delivery.
#[derive(Debug, Error)]
pub enum EmailError {
    #[error("invalid email address: {0}")]
    InvalidAddress(String),
    #[error("email transport error: {0}")]
    Transport(String),
    #[error("email provider rejected request: {0}")]
    Provider(String),
}

#[async_trait::async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, msg: OutboundEmail) -> Result<(), EmailError>;
}

/// Build the configured email sender from environment.
///
/// Selection order (completed in Task 2): Resend → SMTP → LogSink.
/// Unconfigured (or until SMTP/Resend adapters land) → LogSink (AUTH-09).
pub fn build_email_sender_from_env() -> Arc<dyn EmailSender> {
    Arc::new(LogSink)
}
