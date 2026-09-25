//! SSH failed-auth rate limit — reuse PAT limiter with fingerprint as user bucket (D-SSH-07).

pub use crate::pat::rate_limit::FailedAuthLimiter;

/// Alias documenting fingerprint bucket reuse of `check_user` / `record_user` / `clear_user`.
pub type SshAuthLimiter = FailedAuthLimiter;
