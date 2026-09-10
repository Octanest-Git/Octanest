//! Email verify (and later reset) token issue/consume — magic+OTP, resend, rate limits.

use chrono::Utc;
use octanest_core::{AppError, UserPublic};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::gate;
use crate::auth::local::user_to_public;
use crate::email::OutboundEmail;
use crate::rpc::RpcCtx;

const PURPOSE_VERIFY: &str = "verify";
const TOKEN_BYTES: usize = 32;
const OTP_DIGITS: u32 = 100_000_000; // 8-digit numeric
const TTL_SECS: i64 = 30 * 60;
const MIN_ISSUE_INTERVAL_SECS: i64 = 60;
const MAX_ISSUES_PER_HOUR: i32 = 5;
const MAX_REDEEM_ATTEMPTS: i32 = 10;
const VERIFY_SUBJECT: &str = "Verify your Octanest email";

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

/// Absolute origin for magic links — `OCTANEST_PUBLIC_ORIGIN` only (T-05-07).
pub fn public_origin() -> String {
    std::env::var("OCTANEST_PUBLIC_ORIGIN")
        .ok()
        .map(|o| o.trim().trim_end_matches('/').to_string())
        .filter(|o| !o.is_empty())
        .unwrap_or_else(|| "http://localhost:8080".into())
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

fn parse_created_at(raw: &str) -> Result<chrono::DateTime<Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| ndt.and_utc())
                .map_err(|_| {
                    AppError::new("auth.internal", "authentication failed")
                })
        })
}

fn rate_limited() -> AppError {
    AppError::new(
        "auth.rate_limited",
        "too many verification emails; try again later",
    )
}

fn invalid_token() -> AppError {
    AppError::new(
        "auth.invalid_token",
        "invalid or expired verification code",
    )
}

/// Issued secrets returned only to callers that need plaintext (tests / email senders).
#[derive(Debug, Clone)]
pub struct IssuedVerifySecrets {
    pub magic: String,
    pub otp: String,
}

/// Enforce soft rate limits and compute the next `issue_count` (D-19).
fn next_issue_count(
    existing: Option<&octanest_db::email_tokens::EmailTokenRow>,
) -> Result<i32, AppError> {
    let Some(row) = existing else {
        return Ok(1);
    };
    let created = parse_created_at(&row.created_at)?;
    let age = Utc::now().signed_duration_since(created);
    if age.num_seconds() < MIN_ISSUE_INTERVAL_SECS {
        return Err(rate_limited());
    }
    if age.num_seconds() >= 3600 {
        return Ok(1);
    }
    if row.issue_count >= MAX_ISSUES_PER_HOUR {
        return Err(rate_limited());
    }
    Ok(row.issue_count + 1)
}

fn build_verify_email(to: &str, username: &str, magic: &str, otp: &str) -> OutboundEmail {
    let origin = public_origin();
    let link = format!("{origin}/verify?token={magic}");
    let text = format!(
        "Hi {username},\n\n\
Verify your Octanest email with this link:\n{link}\n\n\
Or enter this 8-digit code:\n{otp}\n\n\
This link and code expire in 30 minutes.\n\
If you did not create an Octanest account, you can ignore this email.\n"
    );
    OutboundEmail {
        to: to.to_string(),
        subject: VERIFY_SUBJECT.into(),
        text,
        html: None,
    }
}

/// Issue (or replace) a verify token row for `user_id`. Returns plaintext magic + OTP.
///
/// Applies soft rate limits when `enforce_rate_limit` is true (RPC paths).
/// Library/test helpers may pass `false` to seed a known OTP without waiting.
pub async fn issue_verify_inner(
    db: &octanest_db::Database,
    user_id: &str,
    enforce_rate_limit: bool,
) -> Result<IssuedVerifySecrets, AppError> {
    let existing = db
        .find_email_token_by_user_purpose(user_id, PURPOSE_VERIFY)
        .await
        .map_err(db_err)?;
    let issue_count = if enforce_rate_limit {
        next_issue_count(existing.as_ref())?
    } else {
        existing
            .as_ref()
            .map(|r| r.issue_count.saturating_add(1).max(1))
            .unwrap_or(1)
    };

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
        issue_count,
    )
    .await
    .map_err(db_err)?;
    Ok(IssuedVerifySecrets { magic, otp })
}

/// Issue (or replace) a verify token row — no rate limit (tests / internal seed).
pub async fn issue_verify(
    db: &octanest_db::Database,
    user_id: &str,
) -> Result<IssuedVerifySecrets, AppError> {
    issue_verify_inner(db, user_id, false).await
}

/// Issue + send verify email with rate limits (request/resend/signup auto-send).
pub async fn issue_and_send_verify(
    ctx: &RpcCtx,
    user_id: &str,
    email: &str,
    username: &str,
) -> Result<(), AppError> {
    let secrets = issue_verify_inner(&ctx.db, user_id, true).await?;
    let msg = build_verify_email(email, username, &secrets.magic, &secrets.otp);
    if let Err(e) = ctx.email.send(msg).await {
        // Never log plaintext OTP/token (T-05-08).
        tracing::error!(error = %e, "verify email send failed");
    }
    Ok(())
}

async fn require_session_user(ctx: &RpcCtx) -> Result<octanest_db::UserRow, AppError> {
    let Some(session) = &ctx.session else {
        return Err(AppError::new(
            "auth.unauthenticated",
            "not authenticated",
        ));
    };
    ctx.db
        .find_user_by_id(&session.user_id)
        .await
        .map_err(db_err)?
        .ok_or_else(|| AppError::new("auth.unauthenticated", "not authenticated"))
}

/// Signed-in: issue + send verify email (`auth.request_verify`).
pub async fn request_verify(ctx: &RpcCtx) -> Result<serde_json::Value, AppError> {
    let user = require_session_user(ctx).await?;
    issue_and_send_verify(ctx, &user.id, &user.email, &user.username).await?;
    Ok(serde_json::json!({ "ok": true }))
}

/// Signed-in: replace prior issuance + send (`auth.resend_verify`) — same as request (D-19).
pub async fn resend_verify(ctx: &RpcCtx) -> Result<serde_json::Value, AppError> {
    request_verify(ctx).await
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

    let code = req
        .code
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let token = req
        .token
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if code.is_none() && token.is_none() {
        return Err(AppError::new("rpc.bad_input", "provide token or code"));
    }

    // Session-scoped row lookup so failed attempts attribute to this issuance (T-05-06).
    let Some(row) = ctx
        .db
        .find_email_token_by_user_purpose(&session.user_id, PURPOSE_VERIFY)
        .await
        .map_err(db_err)?
    else {
        return Err(invalid_token());
    };

    if row.purpose != PURPOSE_VERIFY {
        return Err(invalid_token());
    }

    let expires_at = chrono::DateTime::parse_from_rfc3339(&row.expires_at)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| invalid_token())?;
    if expires_at <= Utc::now() {
        let _ = ctx.db.delete_email_token(&row.id).await;
        return Err(invalid_token());
    }

    let matches = if let Some(code) = code {
        if code.len() != 8 || !code.chars().all(|c| c.is_ascii_digit()) {
            false
        } else {
            sha256_hex(code.as_bytes()) == row.otp_hash
        }
    } else if let Some(token) = token {
        sha256_hex(token.as_bytes()) == row.token_hash
    } else {
        false
    };

    if !matches {
        let attempts = ctx
            .db
            .increment_email_token_attempts(&row.id)
            .await
            .map_err(db_err)?;
        if attempts >= MAX_REDEEM_ATTEMPTS {
            let _ = ctx.db.delete_email_token(&row.id).await;
        }
        return Err(invalid_token());
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
