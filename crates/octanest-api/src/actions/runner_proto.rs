//! Minimal Actions runner protocol mount (tracer) — JSON Register + FetchTask.
//! Full Connect/prost wiring lands in 19-05; auth + mount path stay stable (D-ACT-07 / D-ACT-18).

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use octanest_db::Database;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/fetch_task", post(fetch_task))
}

fn hash_token(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    hex::encode(h.finalize())
}

/// Prefer hex crate — if missing, use manual encode.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    let raw = auth.strip_prefix("Bearer ").or_else(|| auth.strip_prefix("bearer "))?;
    Some(raw.trim().to_string())
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    name: String,
    labels: Vec<String>,
    /// Registration token (plaintext once).
    token: String,
}

#[derive(Debug, Serialize)]
struct RegisterResponse {
    runner_id: String,
    runner_token: String,
}

async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    // Session cookies must not authorize (D-ACT-18) — only body/header tokens.
    let _ = headers.get(axum::http::header::COOKIE); // intentionally ignored

    let token_hash = hash_token(&req.token);
    let ok = state
        .db
        .consume_action_runner_registration_token(&token_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !ok {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let runner_token = format!("ort_{}", Uuid::new_v4());
    let runner_hash = hash_token(&runner_token);
    let labels = serde_json::to_string(&req.labels).unwrap_or_else(|_| "[]".into());
    let id = Uuid::new_v4().to_string();
    state
        .db
        .insert_action_runner(&id, &req.name, &runner_hash, &labels, None, false)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse {
        runner_id: id,
        runner_token,
    }))
}

#[derive(Debug, Deserialize)]
struct FetchTaskRequest {
    /// Optional; Bearer preferred.
    runner_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct FetchTaskResponse {
    job_id: Option<String>,
    run_id: Option<String>,
    job_key: Option<String>,
    runs_on: Option<Vec<String>>,
}

async fn fetch_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<FetchTaskRequest>,
) -> Result<Json<FetchTaskResponse>, StatusCode> {
    let _ = headers.get(axum::http::header::COOKIE); // ignore session cookies

    let raw = bearer_token(&headers)
        .or(req.runner_token)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let hash = hash_token(&raw);
    let runner = state
        .db
        .find_action_runner_by_token_hash(&hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let labels: Vec<String> =
        serde_json::from_str(&runner.labels_json).unwrap_or_default();

    let job = state
        .db
        .claim_queued_action_job_for_labels(&runner.id, &labels)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(match job {
        Some(j) => {
            let runs_on: Vec<String> = serde_json::from_str(&j.runs_on_json).unwrap_or_default();
            FetchTaskResponse {
                job_id: Some(j.id),
                run_id: Some(j.run_id),
                job_key: Some(j.job_key),
                runs_on: Some(runs_on),
            }
        }
        None => FetchTaskResponse {
            job_id: None,
            run_id: None,
            job_key: None,
            runs_on: None,
        },
    }))
}

/// Create an instance registration token (Admin/tests).
pub async fn mint_registration_token(db: &Database) -> Result<String, String> {
    let raw = format!("reg_{}", Uuid::new_v4());
    let hash = hash_token(&raw);
    db.insert_action_runner_token(&Uuid::new_v4().to_string(), &hash, "instance", None, true)
        .await?;
    Ok(raw)
}

pub fn token_hash_public(raw: &str) -> String {
    hash_token(raw)
}

/// Keep Arc import used for future Connect state.
#[allow(dead_code)]
fn _arc_marker() -> Arc<()> {
    Arc::new(())
}
