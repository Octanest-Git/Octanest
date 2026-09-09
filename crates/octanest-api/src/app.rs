use std::path::PathBuf;
use std::sync::Arc;

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

use crate::auth::local;
use crate::auth::pending::PendingAuthStore;
use crate::auth::session::{SessionService, SESSION_COOKIE_NAME};
use crate::email::{self, EmailSender};
use crate::routes::{auth_callbacks, avatar};
use crate::rpc::{self, CookieChange, RpcCtx, VERSION_HEADER};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub email: Arc<dyn EmailSender>,
    pub uploads_dir: PathBuf,
    pub sessions: SessionService,
    pub pending: PendingAuthStore,
    pub env_name: String,
}

impl AppState {
    pub fn new(
        db: Database,
        email: Arc<dyn EmailSender>,
        env_name: impl Into<String>,
    ) -> Self {
        let env_name = env_name.into();
        Self {
            db,
            email,
            uploads_dir: PathBuf::from("var/uploads"),
            sessions: SessionService::new(env_name.clone()),
            pending: PendingAuthStore::new(),
            env_name,
        }
    }

    pub fn with_uploads_dir(mut self, dir: PathBuf) -> Self {
        self.uploads_dir = dir;
        self
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
    RpcCtx {
        db: state.db.clone(),
        email: state.email.clone(),
        sessions: state.sessions.clone(),
        uploads_dir: state.uploads_dir.clone(),
        env_name: state.env_name.clone(),
        session,
        set_cookie: None,
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
    let cookie = match change {
        CookieChange::Set(c) => c,
        CookieChange::Clear => local::clear_cookie_for_env(env_name),
    };
    let value = cookie.to_string();
    if let Ok(hv) = HeaderValue::from_str(&value) {
        response.headers_mut().append(header::SET_COOKIE, hv);
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
