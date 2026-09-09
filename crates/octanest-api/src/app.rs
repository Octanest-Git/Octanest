use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use octanest_core::{AppError, RpcRequest, RpcResponse};
use octanest_db::Database;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::rpc::{self, VERSION_HEADER};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

pub fn router(db: Database, cors: CorsLayer) -> Router {
    let state = AppState { db };
    Router::new()
        .route("/health", get(health))
        .route("/api/rpc", post(rpc_http))
        .route("/api/rpc/ws", get(rpc_ws))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn rpc_http(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RpcRequest>,
) -> impl IntoResponse {
    let version = headers
        .get(VERSION_HEADER)
        .and_then(|v| v.to_str().ok());
    if let Err(err) = rpc::check_version_header(version) {
        return (StatusCode::BAD_REQUEST, Json(RpcResponse::err(err)));
    }
    let resp = rpc::dispatch(&state.db, body).await;
    let status = match &resp {
        RpcResponse::Ok { .. } => StatusCode::OK,
        RpcResponse::Err { error, .. } if error.code == "rpc.unknown_procedure" => {
            StatusCode::NOT_FOUND
        }
        RpcResponse::Err { .. } => StatusCode::BAD_REQUEST,
    };
    (status, Json(resp))
}

async fn rpc_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let version = headers
        .get(VERSION_HEADER)
        .and_then(|v| v.to_str().ok());
    if let Err(err) = rpc::check_version_header(version) {
        return (
            StatusCode::BAD_REQUEST,
            Json(RpcResponse::err(err)),
        )
            .into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    while let Some(Ok(msg)) = receiver.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };
        let resp = match serde_json::from_str::<RpcRequest>(&text) {
            Ok(req) => rpc::dispatch(&state.db, req).await,
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
