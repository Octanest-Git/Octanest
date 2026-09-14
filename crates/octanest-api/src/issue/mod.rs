//! Issue RPC handlers — lifecycle + comments + history (ISS-01..02 / D-ISS-01..04 / D-ISS-09 / D-ISS-12 / D-ISS-20).

pub(crate) mod acl;

use octanest_core::{
    AppError, AssigneeCandidatesRequest, AssigneeCandidatesResponse, CommentHistoryResponse,
    CommentRevisionPublic, CreateIssueCommentRequest, CreateIssueRequest, DeleteIssueCommentResponse,
    DeleteIssueRequest, DeleteIssueResponse, IssueAssigneePublic, IssueCommentPublic,
    IssueCommentRefRequest, IssueCommentsListResponse, IssueHistoryResponse, IssueListRequest,
    IssueListResponse, IssuePublic, IssueRefRequest, IssueRevisionPublic, IssueState,
    SetIssueAssigneesRequest, SetIssueLabelsRequest, UpdateIssueCommentRequest, UpdateIssueRequest,
};
use octanest_db::{IssueCommentRow, IssueRow};
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
    let label_rows = ctx
        .db
        .list_labels_for_issue(&row.id)
        .await
        .map_err(db_err)?;
    let labels = crate::label::label_rows_to_public(&label_rows);
    let assignee_rows = ctx
        .db
        .list_issue_assignees(&row.id)
        .await
        .map_err(db_err)?;
    let assignees = assignee_rows
        .into_iter()
        .map(|a| IssueAssigneePublic {
            user_id: a.user_id,
            username: a.username,
            display_name: a.display_name,
        })
        .collect();
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
        labels,
        assignees,
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

fn comment_not_found() -> AppError {
    AppError::new("issue.comment_not_found", "Comment not found")
}

async fn comment_to_public(
    ctx: &RpcCtx,
    row: &IssueCommentRow,
) -> Result<IssueCommentPublic, AppError> {
    let author_username = match ctx.db.find_user_by_id(&row.author_id).await {
        Ok(Some(u)) => u.username,
        Ok(None) => String::new(),
        Err(e) => return Err(db_err(e)),
    };
    Ok(IssueCommentPublic {
        id: row.id.clone(),
        issue_id: row.issue_id.clone(),
        author_id: row.author_id.clone(),
        author_username,
        body: row.body.clone(),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    })
}

async fn load_comment_in_issue(
    ctx: &RpcCtx,
    issue_id: &str,
    comment_id: &str,
) -> Result<IssueCommentRow, AppError> {
    let row = ctx
        .db
        .find_issue_comment_by_id(comment_id)
        .await
        .map_err(db_err)?
        .ok_or_else(comment_not_found)?;
    if row.issue_id != issue_id {
        return Err(comment_not_found());
    }
    Ok(row)
}

/// `issue.comments.list` — Read+; oldest-first (ISS-02 / D-ISS-20).
pub async fn comments_list(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<IssueCommentsListResponse, AppError> {
    let req: IssueRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.comments.list input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let issue = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    let rows = ctx
        .db
        .list_issue_comments(&issue.id)
        .await
        .map_err(db_err)?;
    let mut comments = Vec::with_capacity(rows.len());
    for row in &rows {
        comments.push(comment_to_public(ctx, row).await?);
    }
    Ok(IssueCommentsListResponse { comments })
}

/// `issue.comments.create` — Write+ (ISS-02 / D-ISS-20).
pub async fn comments_create(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<IssueCommentPublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: CreateIssueCommentRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.comments.create input: {e}"),
        )
    })?;
    let body = validate_body(Some(req.body.as_str()))?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let issue = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    let id = Uuid::new_v4().to_string();
    let row = ctx
        .db
        .insert_issue_comment(&id, &issue.id, &user.id, &body)
        .await
        .map_err(db_err)?;
    comment_to_public(ctx, &row).await
}

/// `issue.comments.update` — author only; appends full revision (D-ISS-09 / D-ISS-12).
pub async fn comments_update(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<IssueCommentPublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: UpdateIssueCommentRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.comments.update input: {e}"),
        )
    })?;
    let new_body = validate_body(Some(req.body.as_str()))?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let issue = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    let row = load_comment_in_issue(ctx, &issue.id, &req.comment_id).await?;
    if !acl::can_edit_comment(&user.id, &row.author_id) {
        return Err(not_found());
    }
    if new_body == row.body {
        return comment_to_public(ctx, &row).await;
    }
    let rev_id = Uuid::new_v4().to_string();
    ctx.db
        .insert_comment_revision(&rev_id, &row.id, &user.id, &row.body)
        .await
        .map_err(db_err)?;
    let updated = ctx
        .db
        .update_issue_comment_body(&row.id, &new_body)
        .await
        .map_err(db_err)?;
    comment_to_public(ctx, &updated).await
}

/// `issue.comments.delete` — author or Write+ moderation (D-ISS-09 / T-11-10).
pub async fn comments_delete(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<DeleteIssueCommentResponse, AppError> {
    let user = require_verified(ctx).await?;
    let req: IssueCommentRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.comments.delete input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let issue = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    let row = load_comment_in_issue(ctx, &issue.id, &req.comment_id).await?;
    if !acl::can_delete_comment(&user.id, &row.author_id, accessible.capability) {
        return Err(not_found());
    }
    ctx.db
        .delete_issue_comment(&row.id)
        .await
        .map_err(db_err)?;
    Ok(DeleteIssueCommentResponse { ok: true })
}

/// `issue.comments.history` — Read+; prior body revisions oldest-first (D-ISS-12).
pub async fn comments_history(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<CommentHistoryResponse, AppError> {
    let req: IssueCommentRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.comments.history input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let issue = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;
    let row = load_comment_in_issue(ctx, &issue.id, &req.comment_id).await?;
    let revs = ctx
        .db
        .list_comment_revisions(&row.id)
        .await
        .map_err(db_err)?;
    let mut revisions = Vec::with_capacity(revs.len());
    for rev in &revs {
        let editor_username = match ctx.db.find_user_by_id(&rev.editor_id).await {
            Ok(Some(u)) => u.username,
            Ok(None) => String::new(),
            Err(e) => return Err(db_err(e)),
        };
        revisions.push(CommentRevisionPublic {
            id: rev.id.clone(),
            comment_id: rev.comment_id.clone(),
            editor_id: rev.editor_id.clone(),
            editor_username,
            body: rev.body.clone(),
            created_at: rev.created_at.clone(),
        });
    }
    Ok(CommentHistoryResponse { revisions })
}

/// `issue.labels.set` — Write+; replace label ids from effective repo set (D-ISS-07 / D-ISS-20).
pub async fn labels_set(ctx: &RpcCtx, input: serde_json::Value) -> Result<IssuePublic, AppError> {
    let _user = require_verified(ctx).await?;
    let req: SetIssueLabelsRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.labels.set input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let issue = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;

    let allowed = crate::label::effective_label_id_set(
        ctx,
        &accessible.row.id,
        &accessible.row.owner_type,
        &accessible.row.owner_id,
    )
    .await?;

    let mut seen = std::collections::HashSet::new();
    let mut unique_ids = Vec::new();
    for id in &req.label_ids {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError::new("rpc.bad_input", "label id must not be empty"));
        }
        if !allowed.contains(id) {
            return Err(AppError::new(
                "rpc.bad_input",
                "label is not in the effective set for this repository",
            ));
        }
        if seen.insert(id.to_string()) {
            unique_ids.push(id.to_string());
        }
    }

    ctx.db
        .set_issue_labels(&issue.id, &unique_ids)
        .await
        .map_err(db_err)?;

    let refreshed = ctx
        .db
        .find_issue_by_id(&issue.id)
        .await
        .map_err(db_err)?
        .ok_or_else(issue_not_found)?;
    to_public(ctx, &refreshed).await
}

/// `issue.assignees.set` — Write+; each user_id must have Read+ (D-ISS-06 / D-ISS-08 / T-11-12).
pub async fn assignees_set(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<IssuePublic, AppError> {
    let _user = require_verified(ctx).await?;
    let req: SetIssueAssigneesRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.assignees.set input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let issue = load_issue_in_repo(ctx, &accessible.row.id, req.number).await?;

    let owner_ref = crate::repo::owner_ref_for_repo(&ctx.db, &accessible.row)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "owner_ref_for_repo failed");
            AppError::new("issue.internal", "issue operation failed")
        })?
        .ok_or_else(not_found)?;

    let mut seen = std::collections::HashSet::new();
    let mut unique_ids = Vec::new();
    for id in &req.user_ids {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError::new("rpc.bad_input", "user id must not be empty"));
        }
        if !seen.insert(id.to_string()) {
            continue;
        }
        let Some(_u) = ctx.db.find_user_by_id(id).await.map_err(db_err)? else {
            return Err(AppError::new(
                "rpc.bad_input",
                "assignee user was not found",
            ));
        };
        let cap = crate::repo::effective_capability(
            &ctx.db,
            Some(id),
            &accessible.row,
            &owner_ref,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "effective_capability for assignee failed");
            AppError::new("issue.internal", "issue operation failed")
        })?;
        if !crate::repo::meets(cap, crate::repo::Capability::Read) {
            return Err(AppError::new(
                "rpc.bad_input",
                "assignee must have Read access on this repository",
            ));
        }
        unique_ids.push(id.to_string());
    }

    ctx.db
        .set_issue_assignees(&issue.id, &unique_ids)
        .await
        .map_err(db_err)?;

    let refreshed = ctx
        .db
        .find_issue_by_id(&issue.id)
        .await
        .map_err(db_err)?
        .ok_or_else(issue_not_found)?;
    to_public(ctx, &refreshed).await
}

/// `issue.assigneeCandidates` — Write+; profiles with effective Read+ (D-ISS-08).
pub async fn assignee_candidates(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<AssigneeCandidatesResponse, AppError> {
    let _user = require_verified(ctx).await?;
    let req: AssigneeCandidatesRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid issue.assigneeCandidates input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let owner_ref = crate::repo::owner_ref_for_repo(&ctx.db, &accessible.row)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "owner_ref_for_repo failed");
            AppError::new("issue.internal", "issue operation failed")
        })?
        .ok_or_else(not_found)?;

    let mut candidate_ids = std::collections::HashSet::new();
    match &owner_ref {
        crate::repo::OwnerRef::User { id, .. } => {
            candidate_ids.insert(id.clone());
        }
        crate::repo::OwnerRef::Org { id, .. } => {
            let members = ctx.db.list_org_members(id).await.map_err(db_err)?;
            for m in members {
                candidate_ids.insert(m.user_id);
            }
        }
    }
    let collabs = ctx
        .db
        .list_repo_collaborators(&accessible.row.id)
        .await
        .map_err(db_err)?;
    for c in collabs {
        candidate_ids.insert(c.user_id);
    }

    let prefix = req
        .prefix
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| p.to_ascii_lowercase());

    if let Some(ref p) = prefix {
        if p.chars().count() >= 2 && !p.contains('@') {
            let hits = ctx
                .db
                .list_users_by_username_prefix(p, 20)
                .await
                .map_err(db_err)?;
            for hit in hits {
                if let Some(u) = ctx
                    .db
                    .find_user_by_username(&hit.username)
                    .await
                    .map_err(db_err)?
                {
                    candidate_ids.insert(u.id);
                }
            }
        }
    }

    let mut users = Vec::new();
    for user_id in candidate_ids {
        let cap = crate::repo::effective_capability(
            &ctx.db,
            Some(&user_id),
            &accessible.row,
            &owner_ref,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "effective_capability for candidate failed");
            AppError::new("issue.internal", "issue operation failed")
        })?;
        if !crate::repo::meets(cap, crate::repo::Capability::Read) {
            continue;
        }
        let Some(u) = ctx.db.find_user_by_id(&user_id).await.map_err(db_err)? else {
            continue;
        };
        if let Some(ref p) = prefix {
            if !u.username.to_ascii_lowercase().starts_with(p.as_str()) {
                continue;
            }
        }
        users.push(IssueAssigneePublic {
            user_id: u.id,
            username: u.username,
            display_name: u.display_name,
        });
    }
    users.sort_by(|a, b| a.username.to_ascii_lowercase().cmp(&b.username.to_ascii_lowercase()));
    Ok(AssigneeCandidatesResponse { users })
}
