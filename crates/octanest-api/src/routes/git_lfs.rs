//! Git LFS Batch + basic transfer under `/{owner}/{repo}.git/info/lfs/…` (D-LFS-05..07).
//!
//! Tracer auth: classic PAT with `repo` scope (full matrix in 14-03). Cookie ignored.

use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use octanest_core::{ClassicPatScope, PatKind, CLASSIC_PAT_PREFIX, FINE_GRAINED_PAT_PREFIX};
use octanest_db::{PatRow, RepositoryRow, UserRow};

use crate::app::AppState;
use crate::auth::session::sha256_hex;
use crate::lfs::batch::{
    BatchAction, BatchActions, BatchObjectOut, BatchRequest, BatchResponse, LfsErrorBody,
    LfsObjectError,
};
use crate::lfs::store;
use crate::repo::{
    is_private_visibility, resolve_owner_slug, OwnerRef,
};

const LFS_AUTHENTICATE: &str = r#"Basic realm="Git LFS""#;
const LFS_JSON: &str = "application/vnd.git-lfs+json";

fn lfs_json_headers() -> [(header::HeaderName, HeaderValue); 1] {
    [(
        header::CONTENT_TYPE,
        HeaderValue::from_static(LFS_JSON),
    )]
}

fn unauthorized_lfs() -> Response {
    let mut res = (
        StatusCode::UNAUTHORIZED,
        lfs_json_headers(),
        Json(LfsErrorBody {
            message: "Credentials needed to access LFS".into(),
            request_id: None,
        }),
    )
        .into_response();
    res.headers_mut().insert(
        header::HeaderName::from_static("lfs-authenticate"),
        HeaderValue::from_static(LFS_AUTHENTICATE),
    );
    res
}

fn forbidden_lfs(msg: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        lfs_json_headers(),
        Json(LfsErrorBody {
            message: msg.into(),
            request_id: None,
        }),
    )
        .into_response()
}

fn not_found_lfs(msg: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        lfs_json_headers(),
        Json(LfsErrorBody {
            message: msg.into(),
            request_id: None,
        }),
    )
        .into_response()
}

fn strip_git_suffix(repo_git: &str) -> Option<&str> {
    repo_git.strip_suffix(".git").filter(|n| !n.is_empty())
}

fn parse_basic(headers: &HeaderMap) -> Option<(String, String)> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let encoded = raw
        .strip_prefix("Basic ")
        .or_else(|| raw.strip_prefix("basic "))?;
    let bytes = base64_lite_decode(encoded.trim())?;
    let decoded = String::from_utf8(bytes).ok()?;
    let (user, pass) = decoded.split_once(':')?;
    Some((user.to_string(), pass.to_string()))
}

fn base64_lite_decode(input: &str) -> Option<Vec<u8>> {
    // Reuse the same approach as Smart HTTP via a tiny decoder.
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut buf = [0u8; 4];
    let mut n = 0;
    for &b in bytes {
        if b == b'=' {
            break;
        }
        buf[n] = val(b)?;
        n += 1;
        if n == 4 {
            out.push((buf[0] << 2) | (buf[1] >> 4));
            out.push((buf[1] << 4) | (buf[2] >> 2));
            out.push((buf[2] << 6) | buf[3]);
            n = 0;
        }
    }
    if n == 3 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
        out.push((buf[1] << 4) | (buf[2] >> 2));
    } else if n == 2 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
    }
    Some(out)
}

struct AuthedPat {
    pat: PatRow,
    #[allow(dead_code)]
    owner: UserRow,
}

async fn authenticate_pat(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<AuthedPat>, Response> {
    // D-LFS-09: Cookie must never authorize LFS.
    let _cookie_ignored = headers.get(header::COOKIE);

    let Some((user, pass)) = parse_basic(headers) else {
        return Ok(None);
    };
    let _ = user;
    if !(pass.starts_with(CLASSIC_PAT_PREFIX) || pass.starts_with(FINE_GRAINED_PAT_PREFIX)) {
        return Err(unauthorized_lfs());
    }
    let token_hash = sha256_hex(pass.as_bytes());
    let pat = match state.db.find_pat_by_token_hash(&token_hash).await {
        Ok(Some(p)) if p.revoked_at.is_none() => p,
        Ok(_) => return Err(unauthorized_lfs()),
        Err(e) => {
            tracing::error!(error = %e, "find_pat_by_token_hash");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    let owner = match state.db.find_user_by_id(&pat.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(unauthorized_lfs()),
        Err(e) => {
            tracing::error!(error = %e, "find_user_by_id");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    Ok(Some(AuthedPat { pat, owner }))
}

fn classic_has_repo_scope(pat: &PatRow) -> bool {
    let scopes: Vec<String> = match pat.scopes_json.as_deref().map(serde_json::from_str) {
        Some(Ok(v)) => v,
        _ => return false,
    };
    scopes
        .iter()
        .any(|s| ClassicPatScope::parse(s).ok() == Some(ClassicPatScope::Repo))
}

/// Tracer: classic `repo` scope authorizes upload+download. FG expanded in 14-03.
fn pat_allows_lfs(pat: &PatRow, upload: bool) -> bool {
    let kind = match PatKind::parse(&pat.kind) {
        Ok(k) => k,
        Err(_) => return false,
    };
    match kind {
        PatKind::Classic => classic_has_repo_scope(pat),
        PatKind::FineGrained => {
            // Tracer: treat any non-revoked FG as read; write needs contents write later.
            let _ = upload;
            true
        }
    }
}

struct ResolvedRepo {
    row: RepositoryRow,
    #[allow(dead_code)]
    owner: OwnerRef,
}

async fn resolve_repo(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<ResolvedRepo, Response> {
    let owner_ref = match resolve_owner_slug(&state.db, owner).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(not_found_lfs("Repository not found")),
        Err(e) => {
            tracing::error!(error = %e, "resolve_owner_slug");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    let row = match state
        .db
        .find_repository_by_owner_name(owner_ref.id(), name)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return Err(not_found_lfs("Repository not found")),
        Err(e) => {
            tracing::error!(error = %e, "find_repository");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    Ok(ResolvedRepo {
        row,
        owner: owner_ref,
    })
}

fn object_href(owner: &str, repo_git: &str, oid: &str) -> String {
    format!("/{owner}/{repo_git}/info/lfs/objects/{oid}")
}

/// `POST /{owner}/{repo}.git/info/lfs/objects/batch`
pub async fn batch(
    State(state): State<AppState>,
    AxumPath((owner, repo_git)): AxumPath<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<BatchRequest>,
) -> Response {
    let Some(repo_name) = strip_git_suffix(&repo_git) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let resolved = match resolve_repo(&state, &owner, repo_name).await {
        Ok(r) => r,
        Err(r) => return r,
    };

    let enabled = match state.db.get_repo_lfs_enabled(&resolved.row.id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "get_repo_lfs_enabled");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !enabled {
        return forbidden_lfs("Git LFS is not enabled for this repository");
    }

    let upload = req.operation.eq_ignore_ascii_case("upload");
    let download = req.operation.eq_ignore_ascii_case("download");
    if !upload && !download {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            lfs_json_headers(),
            Json(LfsErrorBody {
                message: format!("unsupported operation: {}", req.operation),
                request_id: None,
            }),
        )
            .into_response();
    }

    let authed = match authenticate_pat(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };

    let is_private = is_private_visibility(&resolved.row.visibility);
    if upload {
        let Some(auth) = &authed else {
            return unauthorized_lfs();
        };
        if !pat_allows_lfs(&auth.pat, true) {
            return forbidden_lfs("Insufficient personal access token scope for LFS upload");
        }
    } else if is_private {
        let Some(auth) = &authed else {
            return unauthorized_lfs();
        };
        if !pat_allows_lfs(&auth.pat, false) {
            return forbidden_lfs("Insufficient personal access token scope for LFS download");
        }
    }

    // Prefer basic when client lists it (or lists nothing).
    let transfer = if req.transfers.is_empty()
        || req.transfers.iter().any(|t| t.eq_ignore_ascii_case("basic"))
    {
        "basic"
    } else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            lfs_json_headers(),
            Json(LfsErrorBody {
                message: "only basic transfer is supported".into(),
                request_id: None,
            }),
        )
            .into_response();
    };

    let mut objects = Vec::with_capacity(req.objects.len());
    for obj in req.objects {
        if store::validate_oid(&obj.oid).is_err() {
            objects.push(BatchObjectOut {
                oid: obj.oid,
                size: obj.size,
                actions: None,
                error: Some(LfsObjectError {
                    code: 422,
                    message: "invalid oid".into(),
                }),
            });
            continue;
        }

        let on_disk = store::object_exists(&state.lfs_dir, &obj.oid).unwrap_or(false);
        let linked = state
            .db
            .has_lfs_link(&resolved.row.id, &obj.oid)
            .await
            .unwrap_or(false);

        if upload {
            if on_disk && linked {
                // Dedup: omit actions so client skips upload (D-LFS-02).
                objects.push(BatchObjectOut {
                    oid: obj.oid,
                    size: obj.size,
                    actions: None,
                    error: None,
                });
            } else {
                let href = object_href(&owner, &repo_git, &obj.oid);
                objects.push(BatchObjectOut {
                    oid: obj.oid,
                    size: obj.size,
                    actions: Some(BatchActions {
                        upload: Some(BatchAction {
                            href,
                            header: None,
                            expires_in: Some(3600),
                        }),
                        download: None,
                        verify: None,
                    }),
                    error: None,
                });
            }
        } else if on_disk {
            let href = object_href(&owner, &repo_git, &obj.oid);
            objects.push(BatchObjectOut {
                oid: obj.oid,
                size: obj.size,
                actions: Some(BatchActions {
                    upload: None,
                    download: Some(BatchAction {
                        href,
                        header: None,
                        expires_in: Some(3600),
                    }),
                    verify: None,
                }),
                error: None,
            });
        } else {
            objects.push(BatchObjectOut {
                oid: obj.oid,
                size: obj.size,
                actions: None,
                error: Some(LfsObjectError {
                    code: 404,
                    message: "Object does not exist".into(),
                }),
            });
        }
    }

    (
        StatusCode::OK,
        lfs_json_headers(),
        Json(BatchResponse {
            transfer: transfer.into(),
            objects,
        }),
    )
        .into_response()
}

/// `PUT /{owner}/{repo}.git/info/lfs/objects/{oid}`
pub async fn put_object(
    State(state): State<AppState>,
    AxumPath((owner, repo_git, oid)): AxumPath<(String, String, String)>,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let Some(repo_name) = strip_git_suffix(&repo_git) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let resolved = match resolve_repo(&state, &owner, repo_name).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    let enabled = state
        .db
        .get_repo_lfs_enabled(&resolved.row.id)
        .await
        .unwrap_or(false);
    if !enabled {
        return forbidden_lfs("Git LFS is not enabled for this repository");
    }

    let authed = match authenticate_pat(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(auth) = authed else {
        return unauthorized_lfs();
    };
    if !pat_allows_lfs(&auth.pat, true) {
        return forbidden_lfs("Insufficient personal access token scope for LFS upload");
    }

    if store::validate_oid(&oid).is_err() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            lfs_json_headers(),
            Json(LfsErrorBody {
                message: "invalid oid".into(),
                request_id: None,
            }),
        )
            .into_response();
    }

    let stream = body.into_data_stream();
    let size = match store::put_stream(&state.lfs_dir, &oid, None, stream).await {
        Ok(n) => n,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                lfs_json_headers(),
                Json(LfsErrorBody {
                    message: e,
                    request_id: None,
                }),
            )
                .into_response();
        }
    };

    if let Err(e) = state.db.upsert_lfs_object(&oid, size as i64).await {
        tracing::error!(error = %e, "upsert_lfs_object");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if let Err(e) = state.db.link_lfs_object(&resolved.row.id, &oid).await {
        tracing::error!(error = %e, "link_lfs_object");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
}

/// `GET /{owner}/{repo}.git/info/lfs/objects/{oid}`
pub async fn get_object(
    State(state): State<AppState>,
    AxumPath((owner, repo_git, oid)): AxumPath<(String, String, String)>,
    headers: HeaderMap,
) -> Response {
    let Some(repo_name) = strip_git_suffix(&repo_git) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let resolved = match resolve_repo(&state, &owner, repo_name).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    let enabled = state
        .db
        .get_repo_lfs_enabled(&resolved.row.id)
        .await
        .unwrap_or(false);
    if !enabled {
        return forbidden_lfs("Git LFS is not enabled for this repository");
    }

    let is_private = is_private_visibility(&resolved.row.visibility);
    let authed = match authenticate_pat(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if is_private {
        let Some(auth) = &authed else {
            return unauthorized_lfs();
        };
        if !pat_allows_lfs(&auth.pat, false) {
            return forbidden_lfs("Insufficient personal access token scope for LFS download");
        }
    }

    if store::validate_oid(&oid).is_err() {
        return not_found_lfs("Object does not exist");
    }

    let bytes = match store::read_object(&state.lfs_dir, &oid).await {
        Ok(b) => b,
        Err(_) => return not_found_lfs("Object does not exist"),
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        bytes,
    )
        .into_response()
}
