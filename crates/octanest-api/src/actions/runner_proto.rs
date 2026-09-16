//! Actions runner protocol mount — Register/Declare/FetchTask/UpdateTask/UpdateLog (D-ACT-07).
//! HTTP+JSON tracer compatible with pinned `proto/runner.proto` (prost types available as `pb`).

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::actions::logs::append_job_log;
use crate::actions::tokens::{accept_registration_token, hash_token};
use crate::app::AppState;

/// Generated protobuf types (Compile via build.rs).
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/octanest.actions.v1.rs"));
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/declare", post(declare))
        .route("/fetch_task", post(fetch_task))
        .route("/update_task", post(update_task))
        .route("/update_log", post(update_log))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let raw = auth
        .strip_prefix("Bearer ")
        .or_else(|| auth.strip_prefix("bearer "))?;
    Some(raw.trim().to_string())
}

async fn runner_from_headers(
    state: &AppState,
    headers: &HeaderMap,
    body_token: Option<&str>,
) -> Result<octanest_db::ActionRunnerRow, StatusCode> {
    let _ = headers.get(axum::http::header::COOKIE); // D-ACT-18: ignore cookies
    let raw = bearer_token(headers)
        .or_else(|| body_token.map(str::to_string))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let hash = hash_token(&raw);
    state
        .db
        .find_action_runner_by_token_hash(&hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    name: String,
    labels: Vec<String>,
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
    let _ = headers.get(axum::http::header::COOKIE);
    let ok = accept_registration_token(&state.db, &req.token)
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
    // Touch prost types so codegen stays linked.
    let _ = pb::RegisterResponse {
        runner_id: id.clone(),
        runner_token: runner_token.clone(),
    };
    Ok(Json(RegisterResponse {
        runner_id: id,
        runner_token,
    }))
}

#[derive(Debug, Deserialize)]
struct DeclareRequest {
    labels: Vec<String>,
    runner_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct OkResponse {
    ok: bool,
}

async fn declare(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<DeclareRequest>,
) -> Result<Json<OkResponse>, StatusCode> {
    let runner = runner_from_headers(&state, &headers, req.runner_token.as_deref()).await?;
    let labels = serde_json::to_string(&req.labels).unwrap_or_else(|_| "[]".into());
    state
        .db
        .update_action_runner_labels(&runner.id, &labels)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(OkResponse { ok: true }))
}

#[derive(Debug, Deserialize)]
struct FetchTaskRequest {
    runner_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct FetchTaskResponse {
    job_id: Option<String>,
    run_id: Option<String>,
    job_key: Option<String>,
    runs_on: Option<Vec<String>>,
    secrets: Option<std::collections::HashMap<String, String>>,
}

async fn fetch_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<FetchTaskRequest>,
) -> Result<Json<FetchTaskResponse>, StatusCode> {
    let runner = runner_from_headers(&state, &headers, req.runner_token.as_deref()).await?;
    let labels: Vec<String> = serde_json::from_str(&runner.labels_json).unwrap_or_default();
    let job = state
        .db
        .claim_queued_action_job_for_labels(&runner.id, &labels)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(match job {
        Some(j) => {
            let runs_on: Vec<String> = serde_json::from_str(&j.runs_on_json).unwrap_or_default();
            let run = state
                .db
                .find_action_run_by_id(&j.run_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let secrets = if let Some(run) = run {
                crate::actions::secrets::decrypted_secrets_for_repo(&state.db, &run.repository_id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            } else {
                std::collections::HashMap::new()
            };
            FetchTaskResponse {
                job_id: Some(j.id),
                run_id: Some(j.run_id),
                job_key: Some(j.job_key),
                runs_on: Some(runs_on),
                secrets: Some(secrets),
            }
        }
        None => FetchTaskResponse {
            job_id: None,
            run_id: None,
            job_key: None,
            runs_on: None,
            secrets: None,
        },
    }))
}

#[derive(Debug, Deserialize)]
struct UpdateTaskRequest {
    job_id: String,
    state: String,
    runner_token: Option<String>,
}

async fn update_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<OkResponse>, StatusCode> {
    let runner = runner_from_headers(&state, &headers, req.runner_token.as_deref()).await?;
    let job = state
        .db
        .find_action_job_by_id(&req.job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if job.runner_id.as_deref() != Some(runner.id.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }
    let status = match req.state.as_str() {
        "in_progress" | "success" | "failure" | "cancelled" => req.state.as_str(),
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    state
        .db
        .update_action_job_status(&job.id, status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let origin = std::env::var("OCTANEST_PUBLIC_ORIGIN").ok();
    if let Err(e) = crate::actions::statuses::publish_from_job_update(
        &state.db,
        &job.id,
        status,
        origin.as_deref(),
    )
    .await
    {
        tracing::warn!(error = %e, job_id = %job.id, "failed to publish job commit status");
    }
    Ok(Json(OkResponse { ok: true }))
}

#[derive(Debug, Deserialize)]
struct UpdateLogRequest {
    run_id: String,
    job_id: String,
    chunk: String,
    runner_token: Option<String>,
}

async fn update_log(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateLogRequest>,
) -> Result<Json<OkResponse>, StatusCode> {
    let runner = runner_from_headers(&state, &headers, req.runner_token.as_deref()).await?;
    let job = state
        .db
        .find_action_job_by_id(&req.job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if job.runner_id.as_deref() != Some(runner.id.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }
    append_job_log(
        &state.actions_log_dir,
        &req.run_id,
        &req.job_id,
        req.chunk.as_bytes(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(OkResponse { ok: true }))
}
