//! Repository RPC handlers (`repo.create` + browse ACL — GIT-01 / GIT-05 / D-23–D-25).

mod acl;
mod collaborators;
mod templates;

pub use acl::{
    can_read_as_owner, effective_capability, fg_all_covers_repo, is_private_visibility, meets,
    not_found, owner_ref_for_repo, resolve_owner_slug, resolve_repo_for_read, AccessibleRepo,
    Capability, OwnerRef,
};
pub use collaborators::{
    add as collaborators_add, list as collaborators_list, remove as collaborators_remove,
    resolve_repo_for_admin, update as collaborators_update,
};

/// Soft size limit for blob preview / raw soft-cap (D-20 / T-07-16).
/// 1 MiB matches GitHub-like soft preview limits.
pub const BLOB_SOFT_MAX_BYTES: usize = 1_048_576;

/// Soft cap for unified patch bytes in commit/compare (aligned with git backend).
pub const DIFF_SOFT_MAX_BYTES: usize = octanest_git::DIFF_SOFT_MAX_BYTES;

use octanest_core::{
    validate_repo_name, AppError, CreateRepoRequest, OwnerType, RepoBlameLine, RepoBlameRequest,
    RepoBlameResponse, RepoBranchCreateRequest, RepoBranchDeleteRequest, RepoBranchMutationResponse,
    RepoBranchRenameRequest, RepoCommitRequest, RepoCommitResponse, RepoCommitSummary,
    RepoCommitsRequest, RepoCommitsResponse, RepoCompareRequest, RepoCompareResponse,
    RepoCreateDefaults, RepoDiffFile, RepoBlobRequest, RepoBlobResponse, RepoGetRequest,
    RepoLfsEnabledResponse, RepoLfsGetEnabledRequest, RepoLfsSetEnabledRequest,
    RepoListByOwnerRequest, RepoListMineResponse, RepoPublic, RepoRefEntry, RepoRefsResponse,
    RepoSoftDeleteRequest,
    RepoSoftDeleteResponse, RepoTreeEntry, RepoTreeRequest, RepoTreeResponse,
    RepoUpdateVisibilityRequest, RepoVisibility,
};
use uuid::Uuid;

use crate::auth::gate::require_verified;
use crate::git::bare_repo_path;
use crate::rpc::RpcCtx;

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new(
            "db.not_configured",
            "no database configured for this instance",
        )
    } else if e.contains("UNIQUE") || e.contains("unique") || e.contains("Duplicate") {
        AppError::new(
            "repo.name_taken",
            "A repository with this name already exists. Choose a different name.",
        )
    } else {
        tracing::error!("repo db error: {e}");
        AppError::new("repo.internal", "repository operation failed")
    }
}

fn map_visibility(v: RepoVisibility) -> &'static str {
    v.as_str()
}

fn to_public(repo: &AccessibleRepo) -> RepoPublic {
    let visibility = RepoVisibility::parse(&repo.row.visibility).unwrap_or(RepoVisibility::Public);
    let owner_type = OwnerType::parse(&repo.row.owner_type).unwrap_or(OwnerType::User);
    RepoPublic {
        id: repo.row.id.clone(),
        owner_id: repo.row.owner_id.clone(),
        owner_type,
        owner_username: repo.owner_username.clone(),
        name: repo.row.name.clone(),
        description: repo.row.description.clone(),
        visibility,
        default_branch: repo.row.default_branch.clone(),
        updated_at: repo.row.updated_at.clone(),
        can_admin: meets(repo.capability, Capability::Admin),
        can_write: meets(repo.capability, Capability::Write),
    }
}

async fn resolve_visibility(
    ctx: &RpcCtx,
    requested: Option<RepoVisibility>,
) -> Result<RepoVisibility, AppError> {
    if let Some(v) = requested {
        return Ok(v);
    }
    // D-08: instance default_visibility; unset column default is public.
    match ctx.db.get_auth_settings().await {
        Ok(settings) => Ok(RepoVisibility::parse(&settings.default_visibility)
            .unwrap_or(RepoVisibility::Public)),
        Err(e) => {
            tracing::warn!(error = %e, "default_visibility lookup failed; using public");
            Ok(RepoVisibility::Public)
        }
    }
}

fn require_session_user(
    ctx: &RpcCtx,
) -> Result<&crate::auth::session::ResolvedSession, AppError> {
    ctx.session.as_ref().ok_or_else(|| {
        AppError::new("auth.unauthenticated", "not authenticated")
    })
}

fn map_git_err(e: octanest_git::GitError) -> AppError {
    match e {
        octanest_git::GitError::NotFound(msg) => AppError::new("repo.path_not_found", msg),
        octanest_git::GitError::InvalidArg(msg) => AppError::new("repo.invalid_ref", msg),
        other => {
            tracing::error!(error = %other, "git backend error");
            AppError::new("repo.git_failed", "git operation failed")
        }
    }
}

/// WR-01: after insert + git failure, soft-delete the row and best-effort remove partial disk path.
async fn compensate_failed_create(ctx: &RpcCtx, repo_id: &str, path: &std::path::Path) {
    if let Err(e) = ctx.db.soft_delete_repository(repo_id).await {
        tracing::error!(
            error = %e,
            repo_id,
            "soft_delete_repository failed during create compensate"
        );
    }
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            // Blocking file (not a dir) or other leftover — try remove_file.
            if let Err(e2) = tokio::fs::remove_file(path).await {
                if e2.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(
                        error = %e,
                        remove_file = %e2,
                        path = %path.display(),
                        "best-effort remove of partial bare path failed"
                    );
                }
            }
        }
    }
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8000).any(|&b| b == 0)
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let mut n = (chunk[0] as u32) << 16;
        if chunk.len() > 1 {
            n |= (chunk[1] as u32) << 8;
        }
        if chunk.len() > 2 {
            n |= chunk[2] as u32;
        }
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// `repo.listMine` — caller's non-deleted repos sorted by updated_at desc (GIT-01 / D-13).
pub async fn list_mine(ctx: &RpcCtx) -> Result<RepoListMineResponse, AppError> {
    let session = require_session_user(ctx)?;
    let user = ctx
        .db
        .find_user_by_id(&session.user_id)
        .await
        .map_err(db_err)?
        .ok_or_else(|| AppError::new("auth.unauthenticated", "not authenticated"))?;

    let rows = ctx
        .db
        .list_repositories_by_owner(&user.id)
        .await
        .map_err(db_err)?;

    let repos = rows
        .into_iter()
        .map(|row| {
            let visibility = RepoVisibility::parse(&row.visibility).unwrap_or(RepoVisibility::Public);
            let owner_type = OwnerType::parse(&row.owner_type).unwrap_or(OwnerType::User);
            RepoPublic {
                id: row.id,
                owner_id: row.owner_id,
                owner_type,
                owner_username: user.username.clone(),
                name: row.name,
                description: row.description,
                visibility,
                default_branch: row.default_branch,
                updated_at: row.updated_at,
                can_admin: true,
                can_write: true,
            }
        })
        .collect();

    Ok(RepoListMineResponse { repos })
}

/// `repo.listByOwner` — ACL-filtered repos under a user/org slug (D-ORG-06 org overview).
/// Public repos are visible to any caller; private only when coalesce grants Read.
pub async fn list_by_owner(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoListMineResponse, AppError> {
    let req: RepoListByOwnerRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.listByOwner input: {e}"),
        )
    })?;
    let owner_slug = req.owner.trim();
    if owner_slug.is_empty() {
        return Err(AppError::new("rpc.bad_input", "owner is required"));
    }

    let owner_ref = match resolve_owner_slug(&ctx.db, owner_slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(RepoListMineResponse { repos: vec![] }),
        Err(e) => {
            tracing::error!(error = %e, "resolve_owner_slug failed");
            return Err(AppError::new("repo.internal", "repository operation failed"));
        }
    };

    let rows = ctx
        .db
        .list_repositories_by_owner(owner_ref.id())
        .await
        .map_err(db_err)?;

    let caller_id = ctx.session.as_ref().map(|s| s.user_id.as_str());
    let mut repos = Vec::with_capacity(rows.len());
    for row in rows {
        let capability = match effective_capability(&ctx.db, caller_id, &row, &owner_ref).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = %e, "effective_capability failed");
                return Err(AppError::new("repo.internal", "repository operation failed"));
            }
        };
        if !meets(capability, Capability::Read) {
            continue;
        }
        repos.push(to_public(&AccessibleRepo {
            row,
            owner_username: owner_ref.slug().to_string(),
            capability,
        }));
    }

    Ok(RepoListMineResponse { repos })
}

/// `repo.createDefaults` — visibility default + stack/gitignore catalogs for `/new`.
pub async fn create_defaults(ctx: &RpcCtx) -> Result<RepoCreateDefaults, AppError> {
    let _ = require_verified(ctx).await?;
    let default_visibility = resolve_visibility(ctx, None).await?;
    Ok(RepoCreateDefaults {
        default_visibility,
        stacks: templates::list_stacks()?,
        gitignores: templates::list_gitignores()?,
    })
}

/// `repo.get` — ACL-safe metadata (D-23–D-25). Anonymous OK for public.
pub async fn get(ctx: &RpcCtx, input: serde_json::Value) -> Result<RepoPublic, AppError> {
    let req: RepoGetRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.get input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    Ok(to_public(&accessible))
}

/// `repo.tree` — `ls_tree` behind ACL; empty repo → `{ empty: true, entries: [] }`.
pub async fn tree(ctx: &RpcCtx, input: serde_json::Value) -> Result<RepoTreeResponse, AppError> {
    let req: RepoTreeRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.tree input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let path = bare_repo_path(&ctx.repos_dir, &accessible.owner_username, &accessible.row.name)?;
    let ref_name = if req.ref_name.trim().is_empty() {
        accessible.row.default_branch.clone()
    } else {
        req.ref_name.trim().to_string()
    };
    let rel = req.path.unwrap_or_default();
    let entries = ctx
        .git
        .ls_tree(&path, &ref_name, &rel)
        .await
        .map_err(map_git_err)?;

    let refs = ctx.git.list_refs(&path).await.map_err(map_git_err)?;
    let empty = refs.is_empty() && entries.is_empty();

    Ok(RepoTreeResponse {
        empty,
        ref_name,
        path: rel.trim_start_matches('/').to_string(),
        entries: entries
            .into_iter()
            .map(|e| RepoTreeEntry {
                mode: e.mode,
                kind: e.kind.as_str().to_string(),
                oid: e.oid,
                name: e.name,
            })
            .collect(),
    })
}

/// `repo.blob` — blob metadata + soft-capped content (D-20).
pub async fn blob(ctx: &RpcCtx, input: serde_json::Value) -> Result<RepoBlobResponse, AppError> {
    let req: RepoBlobRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.blob input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let path = bare_repo_path(&ctx.repos_dir, &accessible.owner_username, &accessible.row.name)?;
    let ref_name = if req.ref_name.trim().is_empty() {
        accessible.row.default_branch.clone()
    } else {
        req.ref_name.trim().to_string()
    };
    let file_path = req.path.trim_start_matches('/').to_string();
    let bytes = ctx
        .git
        .cat_blob(&path, &ref_name, &file_path)
        .await
        .map_err(map_git_err)?;

    let size = bytes.len() as u64;
    let binary = looks_binary(&bytes);
    let truncated = bytes.len() > BLOB_SOFT_MAX_BYTES;
    let preview = if truncated {
        &bytes[..BLOB_SOFT_MAX_BYTES]
    } else {
        &bytes[..]
    };

    let (encoding, content) = if binary {
        ("base64".to_string(), Some(base64_encode(preview)))
    } else {
        match std::str::from_utf8(preview) {
            Ok(s) => ("utf-8".to_string(), Some(s.to_string())),
            Err(_) => ("base64".to_string(), Some(base64_encode(preview))),
        }
    };

    Ok(RepoBlobResponse {
        path: file_path,
        ref_name,
        size,
        truncated,
        is_binary: binary,
        encoding,
        content,
        soft_max_bytes: BLOB_SOFT_MAX_BYTES as u64,
    })
}

/// `repo.refs` — branches + tags (ACL-safe).
pub async fn refs(ctx: &RpcCtx, input: serde_json::Value) -> Result<RepoRefsResponse, AppError> {
    let req: RepoGetRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.refs input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let path = bare_repo_path(&ctx.repos_dir, &accessible.owner_username, &accessible.row.name)?;
    let list = ctx.git.list_refs(&path).await.map_err(map_git_err)?;
    Ok(RepoRefsResponse {
        refs: list
            .into_iter()
            .map(|r| RepoRefEntry {
                name: r.name,
                oid: r.oid,
            })
            .collect(),
    })
}

/// `repo.commits` — paged `git log` behind ACL.
pub async fn commits(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoCommitsResponse, AppError> {
    let req: RepoCommitsRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.commits input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let path = bare_repo_path(&ctx.repos_dir, &accessible.owner_username, &accessible.row.name)?;
    let ref_name = if req.ref_name.trim().is_empty() {
        accessible.row.default_branch.clone()
    } else {
        req.ref_name.trim().to_string()
    };
    let limit = if req.limit == 0 { 30 } else { req.limit.min(100) };
    let commits = ctx
        .git
        .log(&path, &ref_name, req.skip, limit)
        .await
        .map_err(map_git_err)?;
    Ok(RepoCommitsResponse {
        ref_name,
        commits: commits
            .into_iter()
            .map(|c| RepoCommitSummary {
                sha: c.sha,
                short_sha: c.short_sha,
                subject: c.subject,
                author_name: c.author_name,
                author_email: c.author_email,
                authored_at: c.authored_at,
            })
            .collect(),
        skip: req.skip,
        limit,
    })
}

/// `repo.commit` — commit detail + unified patches.
pub async fn commit(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoCommitResponse, AppError> {
    let req: RepoCommitRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.commit input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let path = bare_repo_path(&ctx.repos_dir, &accessible.owner_username, &accessible.row.name)?;
    let detail = ctx
        .git
        .show_commit(&path, req.sha.trim())
        .await
        .map_err(map_git_err)?;
    Ok(RepoCommitResponse {
        sha: detail.sha,
        short_sha: detail.short_sha,
        subject: detail.subject,
        body: detail.body,
        author_name: detail.author_name,
        author_email: detail.author_email,
        authored_at: detail.authored_at,
        parents: detail.parents,
        files: detail
            .files
            .into_iter()
            .map(|f| RepoDiffFile {
                path: f.path,
                status: f.status,
                patch: f.patch,
            })
            .collect(),
        truncated: detail.truncated,
    })
}

/// `repo.compare` — `base...head` unified diff (empty → empty result).
pub async fn compare(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoCompareResponse, AppError> {
    let req: RepoCompareRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.compare input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let path = bare_repo_path(&ctx.repos_dir, &accessible.owner_username, &accessible.row.name)?;
    let result = ctx
        .git
        .diff(&path, req.base.trim(), req.head.trim())
        .await
        .map_err(map_git_err)?;
    Ok(RepoCompareResponse {
        base: result.base,
        head: result.head,
        empty: result.empty,
        truncated: result.truncated,
        files: result
            .files
            .into_iter()
            .map(|f| RepoDiffFile {
                path: f.path,
                status: f.status,
                patch: f.patch,
            })
            .collect(),
    })
}

/// `repo.blame` — per-line blame for a text path.
pub async fn blame(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoBlameResponse, AppError> {
    let req: RepoBlameRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.blame input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let path = bare_repo_path(&ctx.repos_dir, &accessible.owner_username, &accessible.row.name)?;
    let ref_name = if req.ref_name.trim().is_empty() {
        accessible.row.default_branch.clone()
    } else {
        req.ref_name.trim().to_string()
    };
    let file_path = req.path.trim_start_matches('/').to_string();
    let blame = ctx
        .git
        .blame(&path, &ref_name, &file_path)
        .await
        .map_err(map_git_err)?;
    Ok(RepoBlameResponse {
        path: blame.path,
        ref_name: blame.ref_name,
        lines: blame
            .lines
            .into_iter()
            .map(|l| RepoBlameLine {
                sha: l.sha,
                author_name: l.author_name,
                authored_at: l.authored_at,
                line_number: l.line_number,
                content: l.content,
            })
            .collect(),
        truncated: blame.truncated,
    })
}

fn soft_protect_err() -> AppError {
    AppError::new(
        "repo.default_branch_protected",
        "The default branch can't be renamed or deleted.",
    )
}

/// Resolve repo for Write mutate (D-27 branch CRUD / T-10-14).
/// Insufficient capability → identical [`acl::not_found`].
/// Visibility / soft-delete use [`resolve_repo_for_admin`] (Admin) instead.
async fn resolve_repo_for_owner_mutate(
    ctx: &RpcCtx,
    owner: &str,
    name: &str,
) -> Result<AccessibleRepo, AppError> {
    let _ = require_verified(ctx).await?;
    let accessible = resolve_repo_for_read(ctx, owner, name).await?;
    if !meets(accessible.capability, Capability::Write) {
        return Err(acl::not_found());
    }
    Ok(accessible)
}

/// Reject reserved git option tokens used as branch names (CR-02 / D-28).
fn reject_option_like_branch(name: &str) -> Result<(), AppError> {
    let t = name.trim();
    if t.starts_with('-') {
        return Err(AppError::new(
            "repo.invalid_ref",
            format!("invalid branch name: {name}"),
        ));
    }
    Ok(())
}

/// `repo.branchCreate` — owner creates a branch from `start` (GIT-06 / D-27).
pub async fn branch_create(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoBranchMutationResponse, AppError> {
    let req: RepoBranchCreateRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.branchCreate input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_owner_mutate(ctx, &req.owner, &req.name).await?;
    let branch = req.branch.trim();
    if branch.is_empty() {
        return Err(AppError::new("repo.invalid_ref", "branch name required"));
    }
    reject_option_like_branch(branch)?;
    let start = req
        .start
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(accessible.row.default_branch.as_str());
    reject_option_like_branch(start)?;
    let path = bare_repo_path(
        &ctx.repos_dir,
        &accessible.owner_username,
        &accessible.row.name,
    )?;
    ctx.git
        .branch_create(&path, branch, start)
        .await
        .map_err(map_git_err)?;
    Ok(RepoBranchMutationResponse {
        branch: branch.to_string(),
    })
}

/// `repo.branchRename` — owner renames a branch; default soft-protected (D-28).
pub async fn branch_rename(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoBranchMutationResponse, AppError> {
    let req: RepoBranchRenameRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.branchRename input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_owner_mutate(ctx, &req.owner, &req.name).await?;
    let from = req.from.trim();
    let to = req.to.trim();
    if from.is_empty() || to.is_empty() {
        return Err(AppError::new(
            "repo.invalid_ref",
            "from and to branch names required",
        ));
    }
    if from == accessible.row.default_branch {
        return Err(soft_protect_err());
    }
    let path = bare_repo_path(
        &ctx.repos_dir,
        &accessible.owner_username,
        &accessible.row.name,
    )?;
    ctx.git
        .branch_rename(&path, from, to)
        .await
        .map_err(map_git_err)?;
    Ok(RepoBranchMutationResponse {
        branch: to.to_string(),
    })
}

/// `repo.branchDelete` — owner deletes a branch; default soft-protected (D-28).
pub async fn branch_delete(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoBranchMutationResponse, AppError> {
    let req: RepoBranchDeleteRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.branchDelete input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_owner_mutate(ctx, &req.owner, &req.name).await?;
    let branch = req.branch.trim();
    if branch.is_empty() {
        return Err(AppError::new("repo.invalid_ref", "branch name required"));
    }
    if branch == accessible.row.default_branch {
        return Err(soft_protect_err());
    }
    let path = bare_repo_path(
        &ctx.repos_dir,
        &accessible.owner_username,
        &accessible.row.name,
    )?;
    ctx.git
        .branch_delete(&path, branch)
        .await
        .map_err(map_git_err)?;
    Ok(RepoBranchMutationResponse {
        branch: branch.to_string(),
    })
}

/// `repo.updateVisibility` — Admin capability toggles public/private (D-26 / ORG-03).
/// Insufficient capability → soft not_found.
pub async fn update_visibility(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoPublic, AppError> {
    let req: RepoUpdateVisibilityRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.updateVisibility input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let row = ctx
        .db
        .update_repository_visibility(&accessible.row.id, map_visibility(req.visibility))
        .await
        .map_err(db_err)?;
    Ok(to_public(&AccessibleRepo {
        row,
        owner_username: accessible.owner_username,
        capability: Some(Capability::Admin),
    }))
}

/// `repo.lfs.setEnabled` — Admin-only per-repo LFS toggle (D-LFS-10).
pub async fn lfs_set_enabled(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoLfsEnabledResponse, AppError> {
    let req: RepoLfsSetEnabledRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.lfs.setEnabled input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    ctx.db
        .set_repo_lfs_enabled(&accessible.row.id, req.enabled)
        .await
        .map_err(db_err)?;
    Ok(RepoLfsEnabledResponse {
        enabled: req.enabled,
    })
}

/// `repo.lfs.getEnabled` — Read capability may inspect LFS enable status.
pub async fn lfs_get_enabled(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoLfsEnabledResponse, AppError> {
    let req: RepoLfsGetEnabledRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.lfs.getEnabled input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let enabled = ctx
        .db
        .get_repo_lfs_enabled(&accessible.row.id)
        .await
        .map_err(db_err)?;
    Ok(RepoLfsEnabledResponse { enabled })
}

/// `repo.softDelete` — Admin soft-deletes after typed name confirm (D-35). Disk purge deferred.
pub async fn soft_delete(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<RepoSoftDeleteResponse, AppError> {
    let req: RepoSoftDeleteRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.softDelete input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let confirm = req.confirm_name.trim();
    if confirm != accessible.row.name.as_str() {
        return Err(AppError::new(
            "repo.confirm_mismatch",
            "Type the repository name exactly to confirm deletion.",
        ));
    }
    ctx.db
        .soft_delete_repository(&accessible.row.id)
        .await
        .map_err(db_err)?;
    Ok(RepoSoftDeleteResponse {
        name: accessible.row.name,
    })
}

/// Deny create under a foreign owner slug (T-10-06 / A5).
fn create_forbidden() -> AppError {
    AppError::new(
        "repo.create_forbidden",
        "You do not have permission to create a repository under this owner.",
    )
}

/// Resolve create target: omit/self → user; org slug → Owner/Admin only (A5).
async fn resolve_create_owner(
    ctx: &RpcCtx,
    user_id: &str,
    user_username: &str,
    owner_slug: Option<&str>,
) -> Result<(String, OwnerType, String), AppError> {
    let slug = owner_slug.map(str::trim).filter(|s| !s.is_empty());
    let Some(slug) = slug else {
        return Ok((
            user_id.to_string(),
            OwnerType::User,
            user_username.to_string(),
        ));
    };

    if slug.eq_ignore_ascii_case(user_username) {
        return Ok((
            user_id.to_string(),
            OwnerType::User,
            user_username.to_string(),
        ));
    }

    let owner_ref = match resolve_owner_slug(&ctx.db, slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(create_forbidden()),
        Err(e) => {
            tracing::error!(error = %e, "resolve_owner_slug for create failed");
            return Err(AppError::new("repo.internal", "repository operation failed"));
        }
    };

    match owner_ref {
        OwnerRef::User { id, username } => {
            if id == user_id {
                Ok((id, OwnerType::User, username))
            } else {
                Err(create_forbidden())
            }
        }
        OwnerRef::Org { id, slug } => {
            let member = match ctx.db.find_org_member(&id, user_id).await {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!(error = %e, "find_org_member for create failed");
                    return Err(AppError::new("repo.internal", "repository operation failed"));
                }
            };
            let allowed = member
                .as_ref()
                .map(|m| m.role == "owner" || m.role == "admin")
                .unwrap_or(false);
            if !allowed {
                return Err(create_forbidden());
            }
            Ok((id, OwnerType::Org, slug))
        }
    }
}

/// `repo.create` — verified owner creates a public/private repo (DB + bare git + optional seed).
pub async fn create(ctx: &RpcCtx, input: serde_json::Value) -> Result<RepoPublic, AppError> {
    let user = require_verified(ctx).await?;

    let req: CreateRepoRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.create input: {e}"))
    })?;

    validate_repo_name(&req.name).map_err(|msg| AppError::new("repo.invalid_name", msg))?;

    let name = req.name.trim().to_string();
    let description = req
        .description
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();
    let visibility = resolve_visibility(ctx, req.visibility).await?;
    let default_branch = if user.default_branch.trim().is_empty() {
        "main".to_string()
    } else {
        user.default_branch.clone()
    };

    let (owner_id, owner_type, owner_slug) = resolve_create_owner(
        ctx,
        &user.id,
        &user.username,
        req.owner.as_deref(),
    )
    .await?;

    let seed_files =
        templates::assemble_seed_files(&req.stack_id, &req.license_id, &req.gitignore_id)?;

    if ctx
        .db
        .find_repository_by_owner_name(&owner_id, &name)
        .await
        .map_err(db_err)?
        .is_some()
    {
        return Err(AppError::new(
            "repo.name_taken",
            "A repository with this name already exists. Choose a different name.",
        ));
    }

    let id = Uuid::new_v4().to_string();
    let row = ctx
        .db
        .insert_repository(
            &id,
            &owner_id,
            owner_type.as_str(),
            &name,
            map_visibility(visibility),
            &description,
            &default_branch,
        )
        .await
        .map_err(db_err)?;

    let path = bare_repo_path(&ctx.repos_dir, &owner_slug, &name)?;
    if let Err(e) = ctx.git.init_bare(&path, &default_branch).await {
        tracing::error!(
            error = %e,
            path = %path.display(),
            "init_bare failed after DB insert"
        );
        // WR-01: compensate so the name is not permanently occupied.
        compensate_failed_create(ctx, &row.id, &path).await;
        return Err(AppError::new(
            "repo.git_init_failed",
            "failed to initialize repository storage",
        ));
    }

    if !seed_files.is_empty() {
        if let Err(e) = ctx
            .git
            .seed_commit(
                &path,
                &default_branch,
                "Initial commit",
                &seed_files,
            )
            .await
        {
            tracing::error!(
                error = %e,
                path = %path.display(),
                "seed_commit failed after init_bare"
            );
            compensate_failed_create(ctx, &row.id, &path).await;
            return Err(AppError::new(
                "repo.git_seed_failed",
                "failed to seed initial commit from templates",
            ));
        }
    }

    Ok(RepoPublic {
        id: row.id,
        owner_id: row.owner_id,
        owner_type,
        owner_username: owner_slug,
        name: row.name,
        description: row.description,
        visibility,
        default_branch: row.default_branch,
        updated_at: row.updated_at,
        can_admin: true,
        can_write: true,
    })
}
