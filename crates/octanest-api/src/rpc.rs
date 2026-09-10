use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use cookie::Cookie;
use octanest_core::{
    AppError, EchoRequest, EchoResponse, HealthResponse, RpcRequest, RpcResponse, ECHO_MAX_BYTES,
    RPC_PROTOCOL_VERSION,
};
use octanest_db::Database;

use crate::auth::admin;
use crate::auth::local;
use crate::auth::profile;
use crate::auth::session::{ResolvedSession, SessionService};
use crate::auth::verify_reset;
use crate::email::EmailSender;

pub const VERSION_HEADER: &str = "Octanest-RPC-Version";

/// Cookie mutation requested by an RPC handler (attached as Set-Cookie on HTTP).
#[derive(Debug)]
pub enum CookieChange {
    Set(Cookie<'static>),
    Clear,
}

/// Session-aware RPC context (RESEARCH Pattern 1).
pub struct RpcCtx {
    pub db: Database,
    pub email: Arc<dyn EmailSender>,
    /// Shared slot so `admin.auth.update_settings` can rebuild the sender for the process.
    pub email_slot: Arc<RwLock<Arc<dyn EmailSender>>>,
    pub sessions: SessionService,
    pub uploads_dir: PathBuf,
    pub env_name: String,
    pub session: Option<ResolvedSession>,
    pub set_cookie: Option<CookieChange>,
}

pub fn check_version_header(value: Option<&str>) -> Result<(), AppError> {
    match value {
        Some(v) if v.trim() == RPC_PROTOCOL_VERSION.to_string() => Ok(()),
        Some(v) => Err(AppError::new(
            "rpc.version_mismatch",
            format!("expected Octanest-RPC-Version {RPC_PROTOCOL_VERSION}, got {v}"),
        )),
        None => Err(AppError::new(
            "rpc.version_mismatch",
            format!("missing Octanest-RPC-Version header (expected {RPC_PROTOCOL_VERSION})"),
        )),
    }
}

pub async fn dispatch(ctx: &mut RpcCtx, req: RpcRequest) -> RpcResponse {
    match req.procedure.as_str() {
        "system.health" => {
            let database = ctx.db.ping().await.to_string();
            RpcResponse::ok(HealthResponse {
                status: "ok".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                database,
            })
        }
        "system.echo" => {
            let echo: EchoRequest = match serde_json::from_value(req.input) {
                Ok(v) => v,
                Err(e) => {
                    return RpcResponse::err(AppError::new(
                        "rpc.bad_input",
                        format!("invalid echo input: {e}"),
                    ))
                }
            };
            if echo.message.len() > ECHO_MAX_BYTES {
                return RpcResponse::err(AppError::new(
                    "rpc.payload_too_large",
                    format!("echo message exceeds {ECHO_MAX_BYTES} bytes"),
                ));
            }
            RpcResponse::ok(EchoResponse {
                message: echo.message,
            })
        }
        "system.db_probe" => match ctx.db.probe().await {
            Ok(result) => RpcResponse::ok(result),
            Err(e) if e == "database not configured" => RpcResponse::err(AppError::new(
                "db.not_configured",
                "no database configured for this instance",
            )),
            Err(e) => {
                tracing::error!("db probe failed: {e}");
                RpcResponse::err(AppError::new("db.probe_failed", "database probe failed"))
            }
        },
        "auth.signup" => match local::signup(ctx, req.input).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "auth.login" => match local::login(ctx, req.input).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "auth.logout" => match local::logout(ctx).await {
            Ok(()) => RpcResponse::ok(serde_json::json!({ "ok": true })),
            Err(e) => RpcResponse::err(e),
        },
        "auth.logout_all" => match local::logout_all(ctx).await {
            Ok(()) => RpcResponse::ok(serde_json::json!({ "ok": true })),
            Err(e) => RpcResponse::err(e),
        },
        "auth.me" => match local::me(ctx).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "auth.provider_config" => match local::provider_config(ctx).await {
            Ok(cfg) => RpcResponse::ok(cfg),
            Err(e) => RpcResponse::err(e),
        },
        "auth.verify" => match verify_reset::verify(ctx, req.input).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "auth.request_verify" => match verify_reset::request_verify(ctx).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "auth.resend_verify" => match verify_reset::resend_verify(ctx).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "auth.request_password_reset" => {
            match verify_reset::request_password_reset(ctx, req.input).await {
                Ok(v) => RpcResponse::ok(v),
                Err(e) => RpcResponse::err(e),
            }
        },
        "auth.reset_password" => match verify_reset::reset_password(ctx, req.input).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "auth.dev.privileged_ping" => {
            if !verify_reset::privileged_ping_env_allowed(&ctx.env_name) {
                RpcResponse::err(AppError::new(
                    "rpc.unknown_procedure",
                    format!("unknown procedure: {}", req.procedure),
                ))
            } else {
                match verify_reset::privileged_ping(ctx).await {
                    Ok(v) => RpcResponse::ok(v),
                    Err(e) => RpcResponse::err(e),
                }
            }
        }
        "user.get_profile" => match profile::get_profile(ctx).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "user.update_profile" => match profile::update_profile(ctx, req.input).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "admin.auth.get_settings" => match admin::get_settings(ctx).await {
            Ok(settings) => RpcResponse::ok(settings),
            Err(e) => RpcResponse::err(e),
        },
        "admin.auth.update_settings" => match admin::update_settings(ctx, req.input).await {
            Ok(settings) => RpcResponse::ok(settings),
            Err(e) => RpcResponse::err(e),
        },
        other => RpcResponse::err(AppError::new(
            "rpc.unknown_procedure",
            format!("unknown procedure: {other}"),
        )),
    }
}
