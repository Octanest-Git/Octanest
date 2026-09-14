use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use axum::extract::DefaultBodyLimit;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use octanest_core::{AppError, RpcRequest, RpcResponse};
use octanest_db::Database;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use octanest_git::{CliGitBackend, GitBackend};

use crate::auth::pending::PendingAuthStore;
use crate::auth::session::{
    build_session_presence_cookie, clear_session_cookie, clear_session_presence_cookie,
    SessionService, SESSION_COOKIE_NAME, SESSION_IDLE,
};
use crate::email::{self, EmailSender};
use crate::pat::rate_limit::FailedAuthLimiter;
use crate::routes::{auth_callbacks, avatar, git_smart_http, repo_raw};
use crate::rpc::{self, CookieChange, RpcCtx, VERSION_HEADER};
use crate::user::rate_limit::LookupLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    /// Swappable email sender (rebuilt on admin.auth.update_settings).
    pub email: Arc<RwLock<Arc<dyn EmailSender>>>,
    pub uploads_dir: PathBuf,
    /// Bare repos root (`OCTANEST_REPOS_DIR`, default `var/repos`) — D-30 / D-31.
    pub repos_dir: PathBuf,
    /// Git forge backend — Phase 7 registers [`CliGitBackend`] only (D-32).
    pub git: Arc<dyn GitBackend>,
    pub sessions: SessionService,
    pub pending: PendingAuthStore,
    /// Smart HTTP failed Basic/PAT auth counters (D-26) — per process.
    pub git_auth_limiter: Arc<Mutex<FailedAuthLimiter>>,
    /// `user.lookup` per-session counters (T-10-03) — per process.
    pub lookup_limiter: Arc<Mutex<LookupLimiter>>,
    pub env_name: String,
}

impl AppState {
    pub fn new(
        db: Database,
        email: Arc<dyn EmailSender>,
        env_name: impl Into<String>,
    ) -> Self {
        let env_name = env_name.into();
        let repos_dir = std::env::var("OCTANEST_REPOS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("var/repos"));
        // Absolutize so git ops that run in a temp `-C` worktree (seed push) still
        // resolve the bare remote correctly when OCTANEST_REPOS_DIR is relative.
        let repos_dir = if repos_dir.is_absolute() {
            repos_dir
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .join(repos_dir)
        };
        Self {
            db,
            email: Arc::new(RwLock::new(email)),
            uploads_dir: PathBuf::from("var/uploads"),
            repos_dir,
            git: Arc::new(CliGitBackend::new()) as Arc<dyn GitBackend>,
            sessions: SessionService::new(env_name.clone()),
            pending: PendingAuthStore::new(),
            git_auth_limiter: Arc::new(Mutex::new(FailedAuthLimiter::new())),
            lookup_limiter: Arc::new(Mutex::new(LookupLimiter::new())),
            env_name,
        }
    }

    pub fn with_uploads_dir(mut self, dir: PathBuf) -> Self {
        self.uploads_dir = dir;
        self
    }

    pub fn with_repos_dir(mut self, dir: PathBuf) -> Self {
        self.repos_dir = dir;
        self
    }

    pub fn with_git(mut self, git: Arc<dyn GitBackend>) -> Self {
        self.git = git;
        self
    }

    pub fn current_email(&self) -> Arc<dyn EmailSender> {
        self.email
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| email::build_email_sender_from_env())
    }
}

/// Build router with default email sender from ENV and `OCTANEST_ENV`.
pub fn router(db: Database, cors: CorsLayer) -> Router {
    let env_name = std::env::var("OCTANEST_ENV").unwrap_or_else(|_| "development".into());
    let email = email::build_email_sender_from_env();
    router_with_state(AppState::new(db, email, env_name), cors)
}

pub fn router_with_state(state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/rpc", post(rpc_http))
        .route("/api/rpc/ws", get(rpc_ws))
        .route("/api/auth/workos/start", get(auth_callbacks::workos_start))
        .route(
            "/api/auth/workos/callback",
            get(auth_callbacks::workos_callback),
        )
        .route("/api/auth/oidc/start", get(auth_callbacks::oidc_start))
        .route(
            "/api/auth/oidc/callback",
            get(auth_callbacks::oidc_callback),
        )
        .route(
            "/api/user/avatar",
            post(avatar::upload_avatar).layer(DefaultBodyLimit::max(avatar::AVATAR_MAX_BYTES)),
        )
        .route("/uploads/avatars/{file}", get(avatar::serve_avatar))
        .route(
            "/api/repos/{owner}/{repo}/raw/{ref}/{*path}",
            get(repo_raw::serve_raw),
        )
        .route(
            "/api/repos/{owner}/{repo}/archive/{*archive_file}",
            get(repo_raw::serve_archive),
        )
        // Smart HTTP — D-18/D-22: only on /{owner}/{repo}.git (segment includes .git suffix)
        .route(
            "/{owner}/{repo_git}/info/refs",
            get(git_smart_http::info_refs),
        )
        .route(
            "/{owner}/{repo_git}/git-upload-pack",
            axum::routing::post(git_smart_http::upload_pack),
        )
        .route(
            "/{owner}/{repo_git}/git-receive-pack",
            axum::routing::post(git_smart_http::receive_pack),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

fn session_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        let prefix = format!("{SESSION_COOKIE_NAME}=");
        if let Some(value) = part.strip_prefix(prefix.as_str()) {
            return Some(value.to_string());
        }
    }
    None
}

async fn build_rpc_ctx(state: &AppState, raw_token: Option<&str>) -> RpcCtx {
    let session = match raw_token {
        Some(token) => match state.sessions.resolve(&state.db, token).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "session resolve failed");
                None
            }
        },
        None => None,
    };
    let email = state.current_email();
    RpcCtx {
        db: state.db.clone(),
        email,
        email_slot: state.email.clone(),
        sessions: state.sessions.clone(),
        uploads_dir: state.uploads_dir.clone(),
        repos_dir: state.repos_dir.clone(),
        git: state.git.clone(),
        env_name: state.env_name.clone(),
        session,
        set_cookie: None,
        lookup_limiter: state.lookup_limiter.clone(),
    }
}

fn append_set_cookie(response: &mut axum::response::Response, cookie: &cookie::Cookie<'_>) {
    let value = cookie.to_string();
    if let Ok(hv) = HeaderValue::from_str(&value) {
        response.headers_mut().append(header::SET_COOKIE, hv);
    }
}

fn attach_set_cookie(
    mut response: axum::response::Response,
    change: Option<CookieChange>,
    env_name: &str,
) -> axum::response::Response {
    let Some(change) = change else {
        return response;
    };
    match change {
        CookieChange::Set(c) => {
            let ttl = c
                .max_age()
                .map(|d| std::time::Duration::from_secs(d.whole_seconds().max(0) as u64))
                .unwrap_or(SESSION_IDLE);
            append_set_cookie(&mut response, &c);
            append_set_cookie(
                &mut response,
                &build_session_presence_cookie(ttl, env_name),
            );
        }
        CookieChange::Clear => {
            append_set_cookie(&mut response, &clear_session_cookie(env_name));
            append_set_cookie(&mut response, &clear_session_presence_cookie(env_name));
        }
    }
    response
}

fn rpc_status(resp: &RpcResponse) -> StatusCode {
    match resp {
        RpcResponse::Ok { .. } => StatusCode::OK,
        RpcResponse::Err { error, .. } if error.code == "rpc.unknown_procedure" => {
            StatusCode::NOT_FOUND
        }
        RpcResponse::Err { error, .. } if error.code == "auth.unauthenticated" => {
            StatusCode::UNAUTHORIZED
        }
        RpcResponse::Err { error, .. } if error.code == "admin.forbidden" => {
            StatusCode::FORBIDDEN
        }
        RpcResponse::Err { error, .. } if error.code == "auth.email_unverified" => {
            StatusCode::FORBIDDEN
        }
        RpcResponse::Err { error, .. } if error.code == "repo.create_forbidden" => {
            StatusCode::FORBIDDEN
        }
        RpcResponse::Err { error, .. } if error.code == "repo.not_found" => StatusCode::NOT_FOUND,
        RpcResponse::Err { error, .. } if error.code == "issue.not_found" => StatusCode::NOT_FOUND,
        RpcResponse::Err { error, .. } if error.code == "repo.path_not_found" => {
            StatusCode::NOT_FOUND
        }
        RpcResponse::Err { .. } => StatusCode::BAD_REQUEST,
    }
}

async fn rpc_http(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RpcRequest>,
) -> impl IntoResponse {
    let version = headers.get(VERSION_HEADER).and_then(|v| v.to_str().ok());
    if let Err(err) = rpc::check_version_header(version) {
        return (StatusCode::BAD_REQUEST, Json(RpcResponse::err(err))).into_response();
    }

    let token = session_token_from_headers(&headers);
    let mut ctx = build_rpc_ctx(&state, token.as_deref()).await;
    let resp = rpc::dispatch(&mut ctx, body).await;
    let status = rpc_status(&resp);
    let set_cookie = ctx.set_cookie.take();
    let env_name = ctx.env_name.clone();
    let response = (status, Json(resp)).into_response();
    attach_set_cookie(response, set_cookie, &env_name)
}

async fn rpc_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let version = headers.get(VERSION_HEADER).and_then(|v| v.to_str().ok());
    if let Err(err) = rpc::check_version_header(version) {
        return (StatusCode::BAD_REQUEST, Json(RpcResponse::err(err))).into_response();
    }
    let token = session_token_from_headers(&headers);
    ws.on_upgrade(move |socket| handle_socket(socket, state, token))
}

async fn handle_socket(socket: WebSocket, state: AppState, token: Option<String>) {
    let (mut sender, mut receiver) = socket.split();
    while let Some(Ok(msg)) = receiver.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };
        let resp = match serde_json::from_str::<RpcRequest>(&text) {
            Ok(req) => {
                let mut ctx = build_rpc_ctx(&state, token.as_deref()).await;
                rpc::dispatch(&mut ctx, req).await
            }
            Err(e) => RpcResponse::err(AppError::new(
                "rpc.bad_input",
                format!("invalid rpc frame: {e}"),
            )),
        };
        let payload = serde_json::to_string(&resp).unwrap_or_else(|_| {
            r#"{"ok":false,"error":{"code":"rpc.internal","message":"serialize failed"}}"#.into()
        });
        if sender.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}
