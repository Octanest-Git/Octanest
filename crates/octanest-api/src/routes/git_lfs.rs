//! Git LFS Batch + basic transfer under `/{owner}/{repo}.git/info/lfs/…` (D-LFS-05..07).
//!
//! Auth: PAT Basic only (D-LFS-09); Cookie ignored. ACL via [`crate::lfs::auth`].

use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use octanest_db::RepositoryRow;

use crate::app::AppState;
use crate::lfs::auth::{authorize_lfs, forbidden_lfs};
use crate::lfs::batch::{
    BatchAction, BatchActions, BatchObjectOut, BatchRequest, BatchResponse, LfsErrorBody,
    LfsObjectError,
};
use crate::lfs::store;
use crate::repo::{is_private_visibility, resolve_owner_slug, OwnerRef};

const LFS_JSON: &str = "application/vnd.git-lfs+json";

fn lfs_json_headers() -> [(header::HeaderName, HeaderValue); 1] {
    [(
        header::CONTENT_TYPE,
        HeaderValue::from_static(LFS_JSON),
    )]
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

struct ResolvedRepo {
    row: RepositoryRow,
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

async fn require_lfs_enabled(state: &AppState, repo_id: &str) -> Result<(), Response> {
    let enabled = match state.db.get_repo_lfs_enabled(repo_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "get_repo_lfs_enabled");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    if !enabled {
        return Err(forbidden_lfs("Git LFS is not enabled for this repository"));
    }
    Ok(())
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

    if let Err(r) = require_lfs_enabled(&state, &resolved.row.id).await {
        return r;
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

    if let Err(r) = authorize_lfs(&state, &headers, &resolved.row, &resolved.owner, upload).await
    {
        return r;
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
    if let Err(r) = require_lfs_enabled(&state, &resolved.row.id).await {
        return r;
    }

    if let Err(r) = authorize_lfs(&state, &headers, &resolved.row, &resolved.owner, true).await {
        return r;
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
    if let Err(r) = require_lfs_enabled(&state, &resolved.row.id).await {
        return r;
    }

    if let Err(r) = authorize_lfs(&state, &headers, &resolved.row, &resolved.owner, false).await {
        return r;
    }
    let _ = is_private_visibility(&resolved.row.visibility);

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
