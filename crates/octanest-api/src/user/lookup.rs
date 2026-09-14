//! `user.lookup` — live username prefix autocomplete (ORG-01 / D-ORG-03 / T-10-03).

use octanest_core::{AppError, UserLookupHit, UserLookupRequest, UserLookupResponse};

use crate::auth::gate::require_verified;
use crate::rpc::RpcCtx;

const LOOKUP_LIMIT: i64 = 10;
const MIN_PREFIX_LEN: usize = 2;

/// `user.lookup` — verified session, rate-limited, username prefix only (never email).
pub async fn lookup(ctx: &RpcCtx, input: serde_json::Value) -> Result<UserLookupResponse, AppError> {
    let _caller = require_verified(ctx).await?;

    let session = ctx.session.as_ref().ok_or_else(|| {
        AppError::new("auth.unauthenticated", "Sign in to look up users.")
    })?;

    {
        let mut lim = ctx
            .lookup_limiter
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if lim.check_and_record(&session.session_id).is_err() {
            return Err(AppError::new(
                "user.rate_limited",
                "Too many lookup requests. Try again shortly.",
            ));
        }
    }

    let req: UserLookupRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid user.lookup input: {e}"))
    })?;

    let prefix = req.prefix.trim();

    // Anti-enumeration: short prefix / email-shaped → empty (no DB search).
    if prefix.chars().count() < MIN_PREFIX_LEN || prefix.contains('@') {
        return Ok(UserLookupResponse { users: vec![] });
    }

    let rows = ctx
        .db
        .list_users_by_username_prefix(prefix, LOOKUP_LIMIT)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "user.lookup db failed");
            AppError::new("user.lookup_failed", "Could not look up users.")
        })?;

    let users = rows
        .into_iter()
        .map(|r| UserLookupHit {
            username: r.username,
            display_name: r.display_name,
            avatar_url: r.avatar_path,
        })
        .collect();

    Ok(UserLookupResponse { users })
}
