//! Email verify (and later reset) token issue/consume — tracer OTP path (D-14, D-18, D-20, D-21).

use chrono::Utc;
use octanest_core::{AppError, UserPublic};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::gate;
use crate::auth::local::user_to_public;
use crate::rpc::RpcCtx;

const PURPOSE_VERIFY: &str = "verify";
const TOKEN_BYTES: usize = 32;
const OTP_DIGITS: u32 = 100_000_000; // 8-digit numeric
const TTL_SECS: i64 = 30 * 60;

/// Env allowlist for `auth.dev.privileged_ping` (D-10 / Open Q2 RESOLVED).
pub fn privileged_ping_env_allowed(env_name: &str) -> bool {
    matches!(
        env_name,
        "development" | "dev" | "test" | "compose"
    ) || cfg!(test)
}

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new(
            "db.not_configured",
            "no database configured for this instance",
        )
    } else {
        tracing::error!("verify/reset db error: {e}");
        AppError::new("auth.internal", "authentication failed")
    }
}

fn sha256_hex(data: &[u8]) -> String {
    bytes_to_hex(&Sha256::digest(data))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

/// CSPRNG 32-byte magic → lowercase hex (64 chars).
fn generate_magic() -> String {
    let mut token_bytes = [0u8; TOKEN_BYTES];
    rand::fill(&mut token_bytes);
    bytes_to_hex(&token_bytes)
}

/// 8-digit numeric OTP via rejection sampling (uniform over 00000000..=99999999).
fn generate_otp() -> String {
    let limit = (u32::MAX / OTP_DIGITS) * OTP_DIGITS;
    loop {
        let mut buf = [0u8; 4];
        rand::fill(&mut buf);
        let n = u32::from_le_bytes(buf);
        if n < limit {
            return format!("{:08}", n % OTP_DIGITS);
        }
    }
}

/// Issued secrets returned only to callers that need plaintext (tests / email senders).
#[derive(Debug, Clone)]
pub struct IssuedVerifySecrets {
    pub magic: String,
    pub otp: String,
}

/// Issue (or replace) a verify token row for `user_id`. Returns plaintext magic + OTP.
pub async fn issue_verify(
    db: &octanest_db::Database,
    user_id: &str,
) -> Result<IssuedVerifySecrets, AppError> {
    let magic = generate_magic();
    let otp = generate_otp();
    let token_hash = sha256_hex(magic.as_bytes());
    let otp_hash = sha256_hex(otp.as_bytes());
    let expires_at = (Utc::now() + chrono::Duration::seconds(TTL_SECS))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let id = Uuid::new_v4().to_string();
    db.upsert_email_token(
        &id,
        user_id,
        PURPOSE_VERIFY,
        &token_hash,
        &otp_hash,
        &expires_at,
    )
    .await
    .map_err(db_err)?;
    Ok(IssuedVerifySecrets { magic, otp })
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

/// Consume verify magic token or OTP while signed in as the target user (D-21).
pub async fn verify(ctx: &RpcCtx, input: serde_json::Value) -> Result<UserPublic, AppError> {
    let Some(session) = &ctx.session else {
        return Err(AppError::new(
            "auth.unauthenticated",
            "not authenticated",
        ));
    };

    let req: VerifyRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid verify input: {e}"))
    })?;

    let token_row = if let Some(code) = req.code.as_deref().map(str::trim).filter(|s| !s.is_empty())
    {
        if code.len() != 8 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::new(
                "auth.invalid_token",
                "invalid or expired verification code",
            ));
        }
        let otp_hash = sha256_hex(code.as_bytes());
        ctx.db
            .find_email_token_by_otp_hash(&otp_hash)
            .await
            .map_err(db_err)?
    } else if let Some(token) = req
        .token
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let token_hash = sha256_hex(token.as_bytes());
        ctx.db
            .find_email_token_by_token_hash(&token_hash)
            .await
            .map_err(db_err)?
    } else {
        return Err(AppError::new(
            "rpc.bad_input",
            "provide token or code",
        ));
    };

    let Some(row) = token_row else {
        return Err(AppError::new(
            "auth.invalid_token",
            "invalid or expired verification code",
        ));
    };

    if row.purpose != PURPOSE_VERIFY {
        return Err(AppError::new(
            "auth.invalid_token",
            "invalid or expired verification code",
        ));
    }

    if row.user_id != session.user_id {
        return Err(AppError::new(
            "auth.invalid_token",
            "invalid or expired verification code",
        ));
    }

    let expires_at = chrono::DateTime::parse_from_rfc3339(&row.expires_at)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| {
            AppError::new(
                "auth.invalid_token",
                "invalid or expired verification code",
            )
        })?;
    if expires_at <= Utc::now() {
        let _ = ctx.db.delete_email_token(&row.id).await;
        return Err(AppError::new(
            "auth.invalid_token",
            "invalid or expired verification code",
        ));
    }

    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    ctx.db
        .set_email_verified_at(&session.user_id, &now)
        .await
        .map_err(db_err)?;
    ctx.db
        .delete_email_token(&row.id)
        .await
        .map_err(db_err)?;

    let user = ctx
        .db
        .find_user_by_id(&session.user_id)
        .await
        .map_err(db_err)?
        .ok_or_else(|| AppError::new("auth.unauthenticated", "not authenticated"))?;
    Ok(user_to_public(&user))
}

/// Dev/test privileged RPC used to prove `require_verified` (D-10).
pub async fn privileged_ping(ctx: &RpcCtx) -> Result<serde_json::Value, AppError> {
    let _user = gate::require_verified(ctx).await?;
    Ok(serde_json::json!({ "ok": true }))
}
