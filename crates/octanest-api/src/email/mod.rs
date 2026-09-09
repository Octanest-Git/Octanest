//! Outbound email adapters (log sink, SMTP, Resend).

mod log_sink;
mod resend;
mod smtp;

pub use log_sink::LogSink;
pub use resend::ResendSender;
pub use smtp::SmtpSender;

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

const DEFAULT_MAIL_FROM: &str = "Octanest <noreply@localhost>";

/// Build the configured email sender from environment.
///
/// Selection order: Resend (`OCTANEST_RESEND_API_KEY`) → SMTP (`OCTANEST_SMTP_URL`) → LogSink.
/// From address: `OCTANEST_MAIL_FROM` (default `Octanest <noreply@localhost>`).
pub fn build_email_sender_from_env() -> Arc<dyn EmailSender> {
    let from = std::env::var("OCTANEST_MAIL_FROM").unwrap_or_else(|_| DEFAULT_MAIL_FROM.into());

    if let Ok(api_key) = std::env::var("OCTANEST_RESEND_API_KEY") {
        if !api_key.is_empty() {
            return Arc::new(ResendSender::new(api_key, from));
        }
    }

    if let Ok(smtp_url) = std::env::var("OCTANEST_SMTP_URL") {
        if !smtp_url.is_empty() {
            match SmtpSender::from_url(&smtp_url, from.clone()) {
                Ok(sender) => return Arc::new(sender),
                Err(e) => {
                    tracing::error!(
                        target: "octanest.mail",
                        error = %e,
                        "failed to configure SMTP; falling back to log sink"
                    );
                }
            }
        }
    }

    Arc::new(LogSink)
}
