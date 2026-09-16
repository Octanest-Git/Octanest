//! `repo.search` — in-repo code/commits/issues/pulls search (GIT-18 / D-SRCH-*).

use octanest_core::{
    AppError, RepoSearchHit, RepoSearchRequest, RepoSearchResponse, RepoSearchType,
};

use crate::git::bare_repo_path;
use crate::rpc::RpcCtx;

use super::{map_git_err, resolve_repo_for_read};

/// Default soft max matches for code search when limit is large (D-SRCH-08).
pub const DEFAULT_SEARCH_MAX_MATCHES: u32 = 100;

/// `repo.search` — permission-aware in-repo search.
pub async fn search(ctx: &RpcCtx, input: serde_json::Value) -> Result<RepoSearchResponse, AppError> {
    let req: RepoSearchRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid repo.search input: {e}"))
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    let limit = req.limit.clamp(1, 100);
    let offset = req.offset;
    let q = req.q.trim().to_string();

    let (hits, truncated) = match req.search_type {
        RepoSearchType::Code => search_code(ctx, &accessible, &req, &q, offset, limit).await?,
        // Tracer: other types accepted with empty hits until plan 16-02.
        RepoSearchType::Commits | RepoSearchType::Issues | RepoSearchType::Pulls => {
            (Vec::new(), false)
        }
    };

    Ok(RepoSearchResponse {
        search_type: req.search_type,
        q,
        hits,
        truncated,
        offset,
        limit,
    })
}

async fn search_code(
    ctx: &RpcCtx,
    accessible: &super::AccessibleRepo,
    req: &RepoSearchRequest,
    q: &str,
    offset: u32,
    limit: u32,
) -> Result<(Vec<RepoSearchHit>, bool), AppError> {
    if q.is_empty() {
        return Ok((Vec::new(), false));
    }
    let path = bare_repo_path(
        &ctx.repos_dir,
        &accessible.owner_username,
        &accessible.row.name,
    )?;
    let ref_name = match req.ref_name.as_deref() {
        Some(r) if !r.trim().is_empty() => r.trim().to_string(),
        _ => accessible.row.default_branch.clone(),
    };
    let soft_cap = DEFAULT_SEARCH_MAX_MATCHES;
    let fetch = offset
        .saturating_add(limit)
        .saturating_add(1)
        .min(soft_cap.saturating_add(1));
    let result = ctx
        .git
        .grep(&path, &ref_name, q, None, fetch)
        .await
        .map_err(map_git_err)?;

    let truncated = result.truncated || result.hits.len() as u32 > offset.saturating_add(limit);
    let page: Vec<RepoSearchHit> = result
        .hits
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|h| RepoSearchHit::Code {
            path: h.path,
            line: h.line,
            content: h.content,
        })
        .collect();
    Ok((page, truncated))
}
