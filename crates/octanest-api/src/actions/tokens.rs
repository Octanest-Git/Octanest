//! Registration token helpers (D-ACT-08).

use octanest_db::Database;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn hash_token(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Mint instance-scoped registration token (Admin / tests).
pub async fn mint_registration_token(db: &Database) -> Result<String, String> {
    let raw = format!("reg_{}", Uuid::new_v4());
    let hash = hash_token(&raw);
    db.insert_action_runner_token(&Uuid::new_v4().to_string(), &hash, "instance", None, true)
        .await?;
    Ok(raw)
}

/// Accept either DB registration token or `OCTANEST_RUNNER_REGISTRATION_TOKEN` bootstrap.
pub async fn accept_registration_token(db: &Database, raw: &str) -> Result<bool, String> {
    if let Ok(env_tok) = std::env::var("OCTANEST_RUNNER_REGISTRATION_TOKEN") {
        if !env_tok.is_empty() && env_tok == raw {
            return Ok(true);
        }
    }
    let hash = hash_token(raw);
    db.consume_action_runner_registration_token(&hash).await
}
