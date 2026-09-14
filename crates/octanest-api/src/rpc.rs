use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use cookie::Cookie;
use octanest_core::{
    AppError, EchoRequest, EchoResponse, HealthResponse, RpcRequest, RpcResponse, ECHO_MAX_BYTES,
    RPC_PROTOCOL_VERSION,
};
use octanest_db::Database;
use octanest_git::GitBackend;

use crate::auth::admin;
use crate::auth::bootstrap;
use crate::auth::local;
use crate::auth::profile;
use crate::auth::session::{ResolvedSession, SessionService};
use crate::auth::verify_reset;
use crate::email::EmailSender;
use crate::org;
use crate::pat;
use crate::repo;
use crate::user;
use crate::user::rate_limit::LookupLimiter;

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
    pub repos_dir: PathBuf,
    pub git: Arc<dyn GitBackend>,
    pub env_name: String,
    pub session: Option<ResolvedSession>,
    pub set_cookie: Option<CookieChange>,
    /// Per-session `user.lookup` rate limiter (T-10-03).
    pub lookup_limiter: Arc<Mutex<LookupLimiter>>,
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
    // D-11 / T-06-06: empty-instance lock — only bootstrap_* + health until setup completes.
    // confirm_admin_credentials stays off the list (ENV path already has users).
    match bootstrap::needs_setup(&ctx.db).await {
        Ok(true) => {
            let allowed = matches!(
                req.procedure.as_str(),
                "auth.bootstrap_status" | "auth.bootstrap_setup" | "system.health"
            );
            if !allowed {
                return RpcResponse::err(AppError::new(
                    "auth.setup_required",
                    "Complete instance setup before continuing.",
                ));
            }
        }
        Ok(false) => {}
        Err(e) => return RpcResponse::err(e),
    }

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
        "auth.bootstrap_status" => match bootstrap::bootstrap_status(ctx).await {
            Ok(status) => RpcResponse::ok(status),
            Err(e) => RpcResponse::err(e),
        },
        "auth.bootstrap_setup" => match bootstrap::bootstrap_setup(ctx, req.input).await {
            Ok(user) => RpcResponse::ok(user),
            Err(e) => RpcResponse::err(e),
        },
        "auth.confirm_admin_credentials" => {
            match bootstrap::confirm_admin_credentials(ctx, req.input).await {
                Ok(user) => RpcResponse::ok(user),
                Err(e) => RpcResponse::err(e),
            }
        }
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
        "user.lookup" => match user::lookup(ctx, req.input).await {
            Ok(list) => RpcResponse::ok(list),
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
        "admin.instance.factory_reset" => match admin::factory_reset(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "admin.repos.gc" => match admin::repo_gc(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "org.create" => match org::create(ctx, req.input).await {
            Ok(org) => RpcResponse::ok(org),
            Err(e) => RpcResponse::err(e),
        },
        "org.get" => match org::get(ctx, req.input).await {
            Ok(org) => RpcResponse::ok(org),
            Err(e) => RpcResponse::err(e),
        },
        "org.listMine" => match org::list_mine(ctx).await {
            Ok(list) => RpcResponse::ok(list),
            Err(e) => RpcResponse::err(e),
        },
        "org.updateSettings" => match org::update_settings(ctx, req.input).await {
            Ok(org) => RpcResponse::ok(org),
            Err(e) => RpcResponse::err(e),
        },
        "org.members.list" => match org::members_list(ctx, req.input).await {
            Ok(list) => RpcResponse::ok(list),
            Err(e) => RpcResponse::err(e),
        },
        "org.members.add" => match org::members_add(ctx, req.input).await {
            Ok(member) => RpcResponse::ok(member),
            Err(e) => RpcResponse::err(e),
        },
        "org.members.updateRole" => match org::members_update_role(ctx, req.input).await {
            Ok(member) => RpcResponse::ok(member),
            Err(e) => RpcResponse::err(e),
        },
        "org.members.remove" => match org::members_remove(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "org.invites.create" => match org::invites_create(ctx, req.input).await {
            Ok(invite) => RpcResponse::ok(invite),
            Err(e) => RpcResponse::err(e),
        },
        "org.invites.list" => match org::invites_list(ctx, req.input).await {
            Ok(list) => RpcResponse::ok(list),
            Err(e) => RpcResponse::err(e),
        },
        "org.invites.revoke" => match org::invites_revoke(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "org.invites.accept" => match org::invites_accept(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "repo.listMine" => match repo::list_mine(ctx).await {
            Ok(list) => RpcResponse::ok(list),
            Err(e) => RpcResponse::err(e),
        },
        "repo.listByOwner" => match repo::list_by_owner(ctx, req.input).await {
            Ok(list) => RpcResponse::ok(list),
            Err(e) => RpcResponse::err(e),
        },
        "repo.createDefaults" => match repo::create_defaults(ctx).await {
            Ok(defaults) => RpcResponse::ok(defaults),
            Err(e) => RpcResponse::err(e),
        },
        "repo.create" => match repo::create(ctx, req.input).await {
            Ok(repo) => RpcResponse::ok(repo),
            Err(e) => RpcResponse::err(e),
        },
        "repo.get" => match repo::get(ctx, req.input).await {
            Ok(repo) => RpcResponse::ok(repo),
            Err(e) => RpcResponse::err(e),
        },
        "repo.tree" => match repo::tree(ctx, req.input).await {
            Ok(tree) => RpcResponse::ok(tree),
            Err(e) => RpcResponse::err(e),
        },
        "repo.blob" => match repo::blob(ctx, req.input).await {
            Ok(blob) => RpcResponse::ok(blob),
            Err(e) => RpcResponse::err(e),
        },
        "repo.refs" => match repo::refs(ctx, req.input).await {
            Ok(refs) => RpcResponse::ok(refs),
            Err(e) => RpcResponse::err(e),
        },
        "repo.commits" => match repo::commits(ctx, req.input).await {
            Ok(commits) => RpcResponse::ok(commits),
            Err(e) => RpcResponse::err(e),
        },
        "repo.commit" => match repo::commit(ctx, req.input).await {
            Ok(commit) => RpcResponse::ok(commit),
            Err(e) => RpcResponse::err(e),
        },
        "repo.compare" => match repo::compare(ctx, req.input).await {
            Ok(compare) => RpcResponse::ok(compare),
            Err(e) => RpcResponse::err(e),
        },
        "repo.blame" => match repo::blame(ctx, req.input).await {
            Ok(blame) => RpcResponse::ok(blame),
            Err(e) => RpcResponse::err(e),
        },
        "repo.branchCreate" => match repo::branch_create(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "repo.branchRename" => match repo::branch_rename(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "repo.branchDelete" => match repo::branch_delete(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "repo.updateVisibility" => match repo::update_visibility(ctx, req.input).await {
            Ok(repo) => RpcResponse::ok(repo),
            Err(e) => RpcResponse::err(e),
        },
        "repo.softDelete" => match repo::soft_delete(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "repo.collaborators.list" => match repo::collaborators_list(ctx, req.input).await {
            Ok(list) => RpcResponse::ok(list),
            Err(e) => RpcResponse::err(e),
        },
        "repo.collaborators.add" => match repo::collaborators_add(ctx, req.input).await {
            Ok(c) => RpcResponse::ok(c),
            Err(e) => RpcResponse::err(e),
        },
        "repo.collaborators.update" => match repo::collaborators_update(ctx, req.input).await {
            Ok(c) => RpcResponse::ok(c),
            Err(e) => RpcResponse::err(e),
        },
        "repo.collaborators.remove" => match repo::collaborators_remove(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "pat.createClassic" => match pat::create_classic(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "pat.createFineGrained" => match pat::create_fine_grained(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "pat.list" => match pat::list(ctx).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        "pat.revoke" => match pat::revoke(ctx, req.input).await {
            Ok(v) => RpcResponse::ok(v),
            Err(e) => RpcResponse::err(e),
        },
        other => RpcResponse::err(AppError::new(
            "rpc.unknown_procedure",
            format!("unknown procedure: {other}"),
        )),
    }
}
