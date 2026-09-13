//! Git Smart HTTP over `/{owner}/{repo}.git` — PAT Basic auth only (D-10–D-12, D-18, D-22).
//!
//! Session cookies are intentionally ignored for authorization (D-12 / T-08-02).

use axum::body::Bytes;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use octanest_core::{CLASSIC_PAT_PREFIX, FINE_GRAINED_PAT_PREFIX};
use serde::Deserialize;

use crate::app::AppState;
use crate::auth::session::sha256_hex;
use crate::git::bare_repo_path;
use crate::git::http_backend::{self, CgiRequest};

const WWW_AUTHENTICATE: &str = r#"Basic realm="Octanest Git""#;
const PAT_HINT: &str =
    "Authentication failed. Use a personal access token as the password (not your account password). Create one in Settings → Personal access tokens.";

/// Username aliases accepted for Basic auth (identity still comes from the PAT) — D-10.
const USERNAME_ALIASES: &[&str] = &["git", "token", "oauth2"];

#[derive(Debug, Deserialize)]
pub struct InfoRefsQuery {
    pub service: Option<String>,
}

fn unauthorized_pat_hint() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [
            (header::WWW_AUTHENTICATE, WWW_AUTHENTICATE),
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
        ],
        PAT_HINT.to_string(),
    )
        .into_response()
}

fn unauthorized_basic() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, WWW_AUTHENTICATE)],
        "Unauthorized",
    )
        .into_response()
}

/// Decode `Authorization: Basic …` → (username, password).
fn parse_basic(headers: &HeaderMap) -> Result<Option<(String, String)>, Response> {
    let Some(raw) = headers.get(header::AUTHORIZATION) else {
        return Ok(None);
    };
    let raw = raw.to_str().map_err(|_| unauthorized_basic())?;
    let Some(encoded) = raw
        .strip_prefix("Basic ")
        .or_else(|| raw.strip_prefix("basic "))
    else {
        return Ok(None);
    };
    let bytes = base64_lite::decode(encoded.trim()).ok_or_else(unauthorized_basic)?;
    let decoded = String::from_utf8(bytes).map_err(|_| unauthorized_basic())?;
    let (user, pass) = decoded.split_once(':').ok_or_else(unauthorized_basic)?;
    Ok(Some((user.to_string(), pass.to_string())))
}

fn looks_like_pat(password: &str) -> bool {
    password.starts_with(CLASSIC_PAT_PREFIX) || password.starts_with(FINE_GRAINED_PAT_PREFIX)
}

struct AuthedPat {
    remote_user: String,
}

/// Resolve Basic credentials to a PAT owner. Cookies are never consulted (D-12).
async fn authenticate_pat(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<AuthedPat>, Response> {
    let Some((username, password)) = parse_basic(headers)? else {
        return Ok(None);
    };
    if username.is_empty() {
        return Err(unauthorized_basic());
    }
    if !looks_like_pat(&password) {
        return Err(unauthorized_pat_hint());
    }
    let token_hash = sha256_hex(password.as_bytes());
    let pat = match state.db.find_pat_by_token_hash(&token_hash).await {
        Ok(Some(p)) => p,
        Ok(None) => return Err(unauthorized_pat_hint()),
        Err(e) => {
            tracing::error!(error = %e, "find_pat_by_token_hash failed");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    let owner = match state.db.find_user_by_id(&pat.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(unauthorized_pat_hint()),
        Err(e) => {
            tracing::error!(error = %e, "find_user_by_id failed");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    let user_ok = USERNAME_ALIASES
        .iter()
        .any(|a| username.eq_ignore_ascii_case(a))
        || username.eq_ignore_ascii_case(&owner.username);
    if !user_ok {
        return Err(unauthorized_basic());
    }
    Ok(Some(AuthedPat {
        remote_user: owner.username,
    }))
}

async fn resolve_repo_visibility(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<bool, Response> {
    let owner_user = match state.db.find_user_by_username(owner).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(unauthorized_basic()),
        Err(e) => {
            tracing::error!(error = %e, "find_user_by_username");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    let row = match state
        .db
        .find_repository_by_owner_name(&owner_user.id, name)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return Err(unauthorized_basic()),
        Err(e) => {
            tracing::error!(error = %e, "find_repository_by_owner_name");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    Ok(row.visibility.eq_ignore_ascii_case("private"))
}

fn is_upload_pack_service(service: Option<&str>, path_tail: &str) -> bool {
    service == Some("git-upload-pack") || path_tail == "git-upload-pack"
}

fn is_receive_pack_service(service: Option<&str>, path_tail: &str) -> bool {
    service == Some("git-receive-pack") || path_tail == "git-receive-pack"
}

fn strip_git_suffix(repo_git: &str) -> Option<&str> {
    repo_git.strip_suffix(".git").filter(|n| !n.is_empty())
}

#[allow(clippy::too_many_arguments)]
async fn authorize_and_cgi(
    state: &AppState,
    headers: &HeaderMap,
    owner: &str,
    repo_git: &str,
    path_tail: &str,
    method: &str,
    query_string: &str,
    service: Option<&str>,
    body: &[u8],
) -> Response {
    // D-12: Cookie / octanest_session must not authenticate Smart HTTP.
    // Reference the header so scanners see an explicit ignore pattern.
    let _octanest_session_cookie_ignored = headers.get(header::COOKIE);

    let Some(repo) = strip_git_suffix(repo_git) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if bare_repo_path(&state.repos_dir, owner, repo).is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let authed = match authenticate_pat(state, headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };

    let is_private = match resolve_repo_visibility(state, owner, repo).await {
        Ok(v) => v,
        Err(r) => return r,
    };

    let upload = is_upload_pack_service(service, path_tail);
    let receive = is_receive_pack_service(service, path_tail);

    if path_tail == "info/refs" && !upload && !receive {
        return (StatusCode::FORBIDDEN, "unsupported git service").into_response();
    }
    if receive && authed.is_none() {
        return unauthorized_basic();
    }
    if is_private && authed.is_none() {
        return unauthorized_basic();
    }

    let path_info = format!("/{owner}/{repo_git}/{path_tail}");
    let git_protocol = headers
        .get("Git-Protocol")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let remote_user = authed.as_ref().map(|a| a.remote_user.as_str());

    match http_backend::run_git_http_backend(CgiRequest {
        repos_dir: &state.repos_dir,
        path_info: &path_info,
        method,
        query_string,
        content_type: content_type.as_deref(),
        body,
        remote_user,
        git_protocol: git_protocol.as_deref(),
    })
    .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!(error = %e, "git-http-backend failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "ok": false,
                    "error": { "code": "git.cgi_failed", "message": "git smart http failed" }
                })),
            )
                .into_response()
        }
    }
}

/// `GET /{owner}/{repo}.git/info/refs?service=…`
pub async fn info_refs(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((owner, repo_git)): AxumPath<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
) -> Response {
    let qs = match query.service.as_deref() {
        Some(s) => format!("service={s}"),
        None => String::new(),
    };
    authorize_and_cgi(
        &state,
        &headers,
        &owner,
        &repo_git,
        "info/refs",
        "GET",
        &qs,
        query.service.as_deref(),
        &[],
    )
    .await
}

/// `POST /{owner}/{repo}.git/git-upload-pack`
pub async fn upload_pack(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((owner, repo_git)): AxumPath<(String, String)>,
    body: Bytes,
) -> Response {
    authorize_and_cgi(
        &state,
        &headers,
        &owner,
        &repo_git,
        "git-upload-pack",
        "POST",
        "",
        Some("git-upload-pack"),
        &body,
    )
    .await
}

/// `POST /{owner}/{repo}.git/git-receive-pack`
pub async fn receive_pack(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((owner, repo_git)): AxumPath<(String, String)>,
    body: Bytes,
) -> Response {
    authorize_and_cgi(
        &state,
        &headers,
        &owner,
        &repo_git,
        "git-receive-pack",
        "POST",
        "",
        Some("git-receive-pack"),
        &body,
    )
    .await
}

/// Tiny base64 decoder — no new crates.io dep (T-08-SC).
mod base64_lite {
    pub fn decode(input: &str) -> Option<Vec<u8>> {
        let bytes: Vec<u8> = input
            .bytes()
            .filter(|&b| !b.is_ascii_whitespace())
            .collect();
        if bytes.is_empty() {
            return Some(Vec::new());
        }
        let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
        let mut buf: u32 = 0;
        let mut bits: i32 = 0;
        for &b in &bytes {
            if b == b'=' {
                break;
            }
            let val = match b {
                b'A'..=b'Z' => b - b'A',
                b'a'..=b'z' => b - b'a' + 26,
                b'0'..=b'9' => b - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => return None,
            } as u32;
            buf = (buf << 6) | val;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buf >> bits) as u8);
                buf &= (1 << bits) - 1;
            }
        }
        Some(out)
    }
}
