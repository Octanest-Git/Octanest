//! Raw blob HTTP (`GET /api/repos/{owner}/{repo}/raw/{ref}/{*path}`) — GIT-05 / D-17.
//!
//! Cookie session ACL via [`crate::repo::resolve_repo_for_read`]. Not RPC JSON (large bytes).

use std::path::{Component, Path};

use axum::extract::{Path as AxumPath, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::auth::session::SESSION_COOKIE_NAME;
use crate::git::bare_repo_path;
use crate::repo::{self, BLOB_SOFT_MAX_BYTES};
use crate::rpc::RpcCtx;

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

fn err_json(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "ok": false,
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

fn validate_ref(ref_name: &str) -> Result<&str, Response> {
    let t = ref_name.trim();
    if t.is_empty() || t.contains('\0') || t.contains("..") || t.contains('/') {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "repo.invalid_ref",
            "invalid ref",
        ));
    }
    Ok(t)
}

fn validate_blob_path(path: &str) -> Result<String, Response> {
    let rel = path.trim().trim_start_matches('/');
    if rel.is_empty() || rel.contains('\0') {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "repo.invalid_path",
            "invalid path",
        ));
    }
    let candidate = Path::new(rel);
    if candidate.is_absolute()
        || candidate.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "repo.invalid_path",
            "path must not contain ..",
        ));
    }
    Ok(rel.to_string())
}

async fn build_ctx(state: &AppState, headers: &HeaderMap) -> RpcCtx {
    let token = session_token_from_headers(headers);
    let session = match token.as_deref() {
        Some(t) => match state.sessions.resolve(&state.db, t).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "session resolve failed on raw");
                None
            }
        },
        None => None,
    };
    let email = state.current_email();
    RpcCtx {
        db: state.db.clone(),
        email: email.clone(),
        email_slot: state.email.clone(),
        sessions: state.sessions.clone(),
        uploads_dir: state.uploads_dir.clone(),
        repos_dir: state.repos_dir.clone(),
        git: state.git.clone(),
        env_name: state.env_name.clone(),
        session,
        set_cookie: None,
    }
}

/// `GET /api/repos/{owner}/{repo}/raw/{ref}/{*path}`
pub async fn serve_raw(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((owner, repo_name, ref_name, path)): AxumPath<(String, String, String, String)>,
) -> Response {
    let ref_name = match validate_ref(&ref_name) {
        Ok(r) => r.to_string(),
        Err(r) => return r,
    };
    let path = match validate_blob_path(&path) {
        Ok(p) => p,
        Err(r) => return r,
    };

    let ctx = build_ctx(&state, &headers).await;
    let accessible = match repo::resolve_repo_for_read(&ctx, &owner, &repo_name).await {
        Ok(a) => a,
        Err(e) if e.code == "repo.not_found" => {
            return err_json(StatusCode::NOT_FOUND, &e.code, &e.message);
        }
        Err(e) => {
            return err_json(StatusCode::BAD_REQUEST, &e.code, &e.message);
        }
    };

    let bare = match bare_repo_path(
        &state.repos_dir,
        &accessible.owner_username,
        &accessible.row.name,
    ) {
        Ok(p) => p,
        Err(e) => return err_json(StatusCode::BAD_REQUEST, &e.code, &e.message),
    };

    let bytes = match state.git.cat_blob(&bare, &ref_name, &path).await {
        Ok(b) => b,
        Err(octanest_git::GitError::NotFound(_)) => {
            return err_json(
                StatusCode::NOT_FOUND,
                "repo.path_not_found",
                "file not found",
            );
        }
        Err(octanest_git::GitError::InvalidArg(msg)) => {
            return err_json(StatusCode::BAD_REQUEST, "repo.invalid_ref", &msg);
        }
        Err(e) => {
            tracing::error!(error = %e, "raw cat_blob failed");
            return err_json(
                StatusCode::INTERNAL_SERVER_ERROR,
                "repo.git_failed",
                "git operation failed",
            );
        }
    };

    let truncated = bytes.len() > BLOB_SOFT_MAX_BYTES;
    let body = if truncated {
        bytes[..BLOB_SOFT_MAX_BYTES].to_vec()
    } else {
        bytes
    };

    let mut res = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        body,
    )
        .into_response();

    if truncated {
        if let Ok(v) = HeaderValue::from_str(&BLOB_SOFT_MAX_BYTES.to_string()) {
            res.headers_mut()
                .insert("x-octanest-blob-truncated", HeaderValue::from_static("1"));
            res.headers_mut().insert("x-octanest-blob-soft-max", v);
        }
    }
    res
}
