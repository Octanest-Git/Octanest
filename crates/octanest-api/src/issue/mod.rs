//! Issue RPC handlers — lifecycle + history (ISS-01 / D-ISS-01..04 / D-ISS-20).

mod acl;

use octanest_core::{
    AppError, CreateIssueRequest, DeleteIssueRequest, DeleteIssueResponse, IssueHistoryResponse,
    IssueListRequest, IssueListResponse, IssuePublic, IssueRefRequest, IssueRevisionPublic,
    IssueState, UpdateIssueRequest,
};
use octanest_db::IssueRow;
use uuid::Uuid;

use crate::auth::gate::require_verified;
use crate::repo::not_found;
use crate::rpc::RpcCtx;

/// Title soft cap (~1k chars).
const TITLE_MAX_CHARS: usize = 1_024;
/// Body soft cap (~64k chars).
const BODY_MAX_CHARS: usize = 65_536;

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new(
            "db.not_configured",
            "no database configured for this instance",
        )
    } else if e.starts_with("invalid issue state filter:") {
        AppError::new("rpc.bad_input", e)
    } else {
        tracing::error!("issue db error: {e}");
        AppError::new("issue.internal", "issue operation failed")
    }
}

fn issue_not_found() -> AppError {
    AppError::new("issue.not_found", "Issue not found")
}

fn validate_title(title: &str) -> Result<&str, AppError> {
    let t = title.trim();
    if t.is_empty() {
        return Err(AppError::new("rpc.bad_input", "title is required"));
    }
    if t.chars().count() > TITLE_MAX_CHARS {
        return Err(AppError::new(
            "rpc.bad_input",
            format!("title exceeds {TITLE_MAX_CHARS} characters"),
        ));
    }
    Ok(t)
}

fn validate_body(body: Option<&str>) -> Result<String, AppError> {
    let stored = body.unwrap_or("").to_string();
    if stored.chars().count() > BODY_MAX_CHARS {
        return Err(AppError::new(
            "rpc.bad_input",
            format!("body exceeds {BODY_MAX_CHARS} characters"),
        ));
    }
    Ok(stored)
}

async fn to_public(ctx: &RpcCtx, row: &IssueRow) -> Result<IssuePublic, AppError> {
    let state = IssueState::parse(&row.state).map_err(|e| {
        tracing::error!(error = %e, "invalid issue state in db");
        AppError::new("issue.internal", "issue operation failed")
    })?;
    let author_username = match ctx.db.find_user_by_id(&row.author_id).await {
        Ok(Some(u)) => u.username,
        Ok(None) => String::new(),
        Err(e) => return Err(db_err(e)),
    };
    Ok(IssuePublic {
        id: row.id.clone(),
        repo_id: row.repo_id.clone(),
        number: row.number,
        title: row.title.clone(),
        body: row.body.clone(),
        state,
        author_id: row.author_id.clone(),
        author_username,
        closed_at: row.closed_at.clone(),
        closed_by: row.closed_by.clone(),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        labels: Vec::new(),
        assignees: Vec::new(),
    })
}

async fn load_issue_in_repo(
    ctx: &RpcCtx,
    repo_id: &str,
    number: i64,
) -> Result<IssueRow, AppError> {
    if number < 1 {
        return Err(issue_not_found());
    }
    ctx.db
        .find_issue_by_repo_number(repo_id, number)
        .await
        .map_err(db_err)?
        .ok_or_else(issue_not_found)
}

/// `issue.create` — Write+; allocates per-repo `#N` (D-ISS-01 / D-ISS-20).
pub async fn create(ctx: &RpcCtx, input: serde_json::Value) -> Result<IssuePublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: CreateIssueRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.create input: {e}"),
        )
    })?;
    let title = validate_title(&req.title)?.to_string();
    let body = validate_body(req.body.as_deref())?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;

    let id = Uuid::new_v4().to_string();
    let row = ctx
        .db
        .insert_issue(&id, &accessible.row.id, &user.id, &title, &body)
        .await
        .map_err(db_err)?;
    to_public(ctx, &row).await
}

/// `issue.get` — Read+; soft `repo.not_found` for unauthorized private (D-ISS-20).
pub async fn get(ctx: &RpcCtx, input: serde_json::Value) -> Result<IssuePublic, AppError> {
    let req: IssueRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid issue.get input: {e}"))
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let row = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    to_public(ctx, &row).await
}

/// `issue.list` — Read+; default state filter `open` (D-ISS-16).
pub async fn list(ctx: &RpcCtx, input: serde_json::Value) -> Result<IssueListResponse, AppError> {
    let req: IssueListRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid issue.list input: {e}"))
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let state = req.state.as_deref().unwrap_or("open");
    let offset = req.offset.unwrap_or(0);
    let limit = req.limit.unwrap_or(30);
    let (rows, total) = ctx
        .db
        .list_issues_for_repo(&accessible.row.id, state, offset, limit)
        .await
        .map_err(db_err)?;
    let mut issues = Vec::with_capacity(rows.len());
    for row in &rows {
        issues.push(to_public(ctx, row).await?);
    }
    Ok(IssueListResponse { issues, total })
}

/// `issue.update` — Author or Write+; appends full revision on title/body change (D-ISS-03 / D-ISS-04).
pub async fn update(ctx: &RpcCtx, input: serde_json::Value) -> Result<IssuePublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: UpdateIssueRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.update input: {e}"),
        )
    })?;
    if req.title.is_none() && req.body.is_none() {
        return Err(AppError::new(
            "rpc.bad_input",
            "title or body is required",
        ));
    }
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let row = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    if !acl::can_edit_issue(&user.id, &row, accessible.capability) {
        return Err(not_found());
    }

    let new_title = match &req.title {
        Some(t) => validate_title(t)?.to_string(),
        None => row.title.clone(),
    };
    let new_body = match &req.body {
        Some(b) => validate_body(Some(b.as_str()))?,
        None => row.body.clone(),
    };

    if new_title == row.title && new_body == row.body {
        return to_public(ctx, &row).await;
    }

    let rev_id = Uuid::new_v4().to_string();
    ctx.db
        .insert_issue_revision(&rev_id, &row.id, &user.id, &row.title, &row.body)
        .await
        .map_err(db_err)?;
    let updated = ctx
        .db
        .update_issue_content(&row.id, &new_title, &new_body)
        .await
        .map_err(db_err)?;
    to_public(ctx, &updated).await
}

/// `issue.close` — Write+; open → closed (D-ISS-02 / D-ISS-20).
pub async fn close(ctx: &RpcCtx, input: serde_json::Value) -> Result<IssuePublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: IssueRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid issue.close input: {e}"))
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let row = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    if row.state == IssueState::Closed.as_str() {
        return to_public(ctx, &row).await;
    }
    let updated = ctx
        .db
        .close_issue(&row.id, &user.id)
        .await
        .map_err(db_err)?;
    to_public(ctx, &updated).await
}

/// `issue.reopen` — Write+; closed → open (D-ISS-02 / D-ISS-20).
pub async fn reopen(ctx: &RpcCtx, input: serde_json::Value) -> Result<IssuePublic, AppError> {
    let _user = require_verified(ctx).await?;
    let req: IssueRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.reopen input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let row = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    if row.state == IssueState::Open.as_str() {
        return to_public(ctx, &row).await;
    }
    let updated = ctx.db.reopen_issue(&row.id).await.map_err(db_err)?;
    to_public(ctx, &updated).await
}

/// `issue.history` — Read+; prior title/body revisions oldest-first (D-ISS-04).
pub async fn history(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<IssueHistoryResponse, AppError> {
    let req: IssueRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.history input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let row = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    let revs = ctx
        .db
        .list_issue_revisions(&row.id)
        .await
        .map_err(db_err)?;
    let mut revisions = Vec::with_capacity(revs.len());
    for rev in &revs {
        let editor_username = match ctx.db.find_user_by_id(&rev.editor_id).await {
            Ok(Some(u)) => u.username,
            Ok(None) => String::new(),
            Err(e) => return Err(db_err(e)),
        };
        revisions.push(IssueRevisionPublic {
            id: rev.id.clone(),
            issue_id: rev.issue_id.clone(),
            editor_id: rev.editor_id.clone(),
            editor_username,
            title: rev.title.clone(),
            body: rev.body.clone(),
            created_at: rev.created_at.clone(),
        });
    }
    Ok(IssueHistoryResponse { revisions })
}

/// `issue.delete` — Admin + confirmNumber; does not reclaim `#N` (D-ISS-02 / D-ISS-20).
pub async fn delete(ctx: &RpcCtx, input: serde_json::Value) -> Result<DeleteIssueResponse, AppError> {
    let _user = require_verified(ctx).await?;
    let req: DeleteIssueRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.delete input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_admin(ctx, &req.owner, &req.name).await?;
    let row = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    if req.confirm_number != row.number {
        return Err(AppError::new(
            "issue.confirm_mismatch",
            "Type the issue number exactly to confirm deletion.",
        ));
    }
    // Cascades comments/reactions/links/revisions via FK ON DELETE CASCADE.
    ctx.db.delete_issue(&row.id).await.map_err(db_err)?;
    Ok(DeleteIssueResponse {
        number: row.number,
    })
}
