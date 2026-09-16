//! Pull request RPC — lifecycle create/get/list/close/reopen (PR-01 / PR-06 / D-PR-02 / D-PR-29).

pub(crate) mod acl;
mod comments;
mod merge_ops;
mod reviews;

pub use comments::{comments_create, comments_list, comments_resolve};
pub use merge_ops::{
    commits, files, merge, merge_settings_get, merge_settings_update,
};
pub use reviews::{
    review_requests_add, review_requests_list, review_requests_remove, reviews_dismiss,
    reviews_list, reviews_submit,
};

use std::path::Path;

use octanest_core::{
    AppError, CreatePullRequest, MergeMethod, PullListRequest, PullListResponse, PullPublic,
    PullRefRequest, PullState, UpdatePullRequest,
};
use octanest_db::{Database, PullRow};
use octanest_git::{CliGitBackend, GitBackend};
use uuid::Uuid;

use crate::auth::gate::require_verified;
use crate::git::bare_repo_path;
use crate::notify;
use crate::protection;
use crate::rpc::RpcCtx;
use crate::webhook::dispatch;
use crate::webhook::payloads;

const TITLE_MAX_CHARS: usize = 1_024;
const BODY_MAX_CHARS: usize = 65_536;

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new(
            "db.not_configured",
            "no database configured for this instance",
        )
    } else {
        tracing::error!("pull db error: {e}");
        AppError::new("pull.internal", "pull operation failed")
    }
}

pub(crate) async fn emit_pull_event(
    ctx: &RpcCtx,
    accessible: &crate::repo::AccessibleRepo,
    row: &PullRow,
    action: &str,
    sender_login: &str,
    sender_id: &str,
    merged: bool,
) {
    emit_pull_event_db(
        &ctx.db,
        &accessible.owner_username,
        &accessible.row.name,
        &accessible.row.id,
        row,
        action,
        sender_login,
        sender_id,
        merged,
        &ctx.env_name,
    )
    .await;
}

/// Emit a `pull_request` webhook from git push paths (no [`RpcCtx`]).
pub(crate) async fn emit_pull_event_db(
    db: &Database,
    base_owner: &str,
    base_name: &str,
    base_repo_id: &str,
    row: &PullRow,
    action: &str,
    sender_login: &str,
    sender_id: &str,
    merged: bool,
    env_name: &str,
) {
    let (head_owner, head_name) = if row.head_repo_id == row.repo_id {
        (base_owner.to_string(), base_name.to_string())
    } else if let Ok(Some(head_repo)) = db.find_repository_by_id(&row.head_repo_id).await {
        let owner = match head_repo.owner_type.as_str() {
            "org" => db
                .find_organization_by_id(&head_repo.owner_id)
                .await
                .ok()
                .flatten()
                .map(|o| o.slug)
                .unwrap_or_default(),
            _ => db
                .find_user_by_id(&head_repo.owner_id)
                .await
                .ok()
                .flatten()
                .map(|u| u.username)
                .unwrap_or_default(),
        };
        (owner, head_repo.name)
    } else {
        (base_owner.to_string(), base_name.to_string())
    };
    let state = if merged { "closed" } else { row.state.as_str() };
    let payload = payloads::pull_request_payload(
        action,
        row.number,
        &row.title,
        &row.body,
        state,
        row.draft,
        merged,
        &row.base_ref,
        &row.head_ref,
        &row.head_sha,
        &head_owner,
        &head_name,
        base_owner,
        base_name,
        base_repo_id,
        sender_login,
        sender_id,
    );
    dispatch::emit(db, base_repo_id, "pull_request", action, payload, env_name).await;
}

async fn resolve_ref_sha_at(
    git: &dyn GitBackend,
    repos_dir: &Path,
    owner: &str,
    repo_name: &str,
    refname: &str,
) -> Option<String> {
    let path = bare_repo_path(repos_dir, owner, repo_name).ok()?;
    let refs = git.list_refs(&path).await.ok()?;
    let want = if refname.starts_with("refs/") {
        refname.to_string()
    } else {
        format!("refs/heads/{refname}")
    };
    refs.iter()
        .find(|r| r.name == want || r.name.ends_with(&format!("/{refname}")) || r.oid == refname)
        .map(|r| r.oid.clone())
}

/// After a successful push: update open same-repo PR heads, dismiss stale approvals, emit synchronize.
pub async fn synchronize_after_push(
    db: &Database,
    repos_dir: &Path,
    repository_id: &str,
    owner: &str,
    repo_name: &str,
    pusher_login: &str,
    pusher_id: &str,
    updates: &[(String, String, String)],
    env_name: &str,
) {
    let git = CliGitBackend::new();
    let mut branch_after: Vec<(String, String)> = updates
        .iter()
        .filter_map(|(_before, after, refname)| {
            let branch = protection::branch_from_ref(refname)?;
            if after.chars().all(|c| c == '0') {
                return None;
            }
            Some((branch.to_string(), after.clone()))
        })
        .collect();

    // SSH notify_push often has no pkt-line updates — refresh open same-repo heads from live refs.
    if branch_after.is_empty() {
        let (open, _) = match db
            .list_pulls_for_repo(repository_id, Some("open"), 0, 500)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "synchronize_after_push: list open pulls failed");
                return;
            }
        };
        for pull in open
            .into_iter()
            .filter(|p| p.head_repo_id == repository_id)
        {
            if let Some(sha) =
                resolve_ref_sha_at(&git, repos_dir, owner, repo_name, &pull.head_ref).await
            {
                if sha != pull.head_sha {
                    branch_after.push((pull.head_ref.clone(), sha));
                }
            }
        }
        // Dedupe by branch name (last wins).
        let mut seen = std::collections::BTreeMap::<String, String>::new();
        for (b, s) in branch_after.drain(..) {
            seen.insert(b, s);
        }
        branch_after = seen.into_iter().collect();
    }

    for (branch, after_sha) in branch_after {
        if let Err(e) = sync_open_pulls_for_branch(
            db,
            repos_dir,
            &git,
            repository_id,
            owner,
            repo_name,
            &branch,
            &after_sha,
            pusher_login,
            pusher_id,
            env_name,
        )
        .await
        {
            tracing::warn!(
                error = %e,
                %branch,
                "synchronize_after_push: branch sync failed (soft-fail)"
            );
        }
    }
}

async fn sync_open_pulls_for_branch(
    db: &Database,
    repos_dir: &Path,
    git: &dyn GitBackend,
    repository_id: &str,
    owner: &str,
    repo_name: &str,
    branch: &str,
    after_sha: &str,
    pusher_login: &str,
    pusher_id: &str,
    env_name: &str,
) -> Result<(), String> {
    let (open, _) = db
        .list_pulls_for_repo(repository_id, Some("open"), 0, 500)
        .await?;
    let matching: Vec<PullRow> = open
        .into_iter()
        .filter(|p| p.head_repo_id == repository_id && p.head_ref == branch)
        .collect();
    if matching.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    for pull in matching {
        if pull.head_sha == after_sha {
            continue;
        }
        let base_sha = resolve_ref_sha_at(git, repos_dir, owner, repo_name, &pull.base_ref)
            .await
            .unwrap_or_else(|| pull.base_sha.clone());
        let base_changed = base_sha != pull.base_sha;
        if base_changed {
            db.update_pull_fields(
                &pull.id,
                &pull.title,
                &pull.body,
                pull.draft,
                &pull.base_ref,
                &base_sha,
            )
            .await?;
        }
        db.update_pull_head_sha(&pull.id, after_sha).await?;
        let _ = db.mark_pull_line_comments_outdated(&pull.id).await;

        let eff = protection::effective_for_branch(db, repository_id, &pull.base_ref)
            .await
            .map_err(|e| e.message)?;
        if eff.dismiss_stale_reviews {
            if let Ok(reviews) = db.list_pull_reviews(&pull.id).await {
                for r in reviews {
                    if r.state == "approved" && r.commit_sha.as_deref() != Some(after_sha) {
                        let _ = db
                            .dismiss_pull_review(
                                &r.id,
                                Some("New commits pushed to the head branch"),
                                &now,
                            )
                            .await;
                    }
                }
            }
        }

        let updated = db
            .find_pull_by_repo_number(repository_id, pull.number)
            .await?
            .ok_or_else(|| "pull missing after synchronize".to_string())?;
        emit_pull_event_db(
            db,
            owner,
            repo_name,
            repository_id,
            &updated,
            "synchronize",
            pusher_login,
            pusher_id,
            false,
            env_name,
        )
        .await;
    }
    Ok(())
}

fn pull_not_found() -> AppError {
    AppError::new("pull.not_found", "Pull request not found")
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

fn validate_ref_name(name: &str, field: &str) -> Result<String, AppError> {
    let t = name.trim();
    if t.is_empty() {
        return Err(AppError::new(
            "rpc.bad_input",
            format!("{field} is required"),
        ));
    }
    if t.contains('\0') || t.contains("..") {
        return Err(AppError::new(
            "rpc.bad_input",
            format!("invalid {field}"),
        ));
    }
    Ok(t.to_string())
}

async fn resolve_ref_sha(
    ctx: &RpcCtx,
    owner: &str,
    repo_name: &str,
    refname: &str,
) -> Result<String, AppError> {
    let path = bare_repo_path(&ctx.repos_dir, owner, repo_name)?;
    let refs = ctx
        .git
        .list_refs(&path)
        .await
        .map_err(|e| AppError::new("pull.ref_not_found", format!("could not list refs: {e}")))?;
    let want = if refname.starts_with("refs/") {
        refname.to_string()
    } else {
        format!("refs/heads/{refname}")
    };
    refs.iter()
        .find(|r| r.name == want || r.name.ends_with(&format!("/{refname}")) || r.oid == refname)
        .map(|r| r.oid.clone())
        .or_else(|| {
            // Allow full SHA if present via show_commit later — try exact oid match length.
            if refname.len() >= 7 && refname.chars().all(|c| c.is_ascii_hexdigit()) {
                Some(refname.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| AppError::new("pull.ref_not_found", format!("ref not found: {refname}")))
}

async fn to_public(ctx: &RpcCtx, row: &PullRow) -> Result<PullPublic, AppError> {
    let state = PullState::parse(&row.state).map_err(|e| {
        tracing::error!(error = %e, "invalid pull state in db");
        AppError::new("pull.internal", "pull operation failed")
    })?;
    let merge_method = match row.merge_method.as_deref() {
        Some(m) => Some(MergeMethod::parse(m).map_err(|e| {
            tracing::error!(error = %e, "invalid merge method in db");
            AppError::new("pull.internal", "pull operation failed")
        })?),
        None => None,
    };
    let author_username = match ctx.db.find_user_by_id(&row.author_id).await {
        Ok(Some(u)) => u.username,
        Ok(None) => String::new(),
        Err(e) => return Err(db_err(e)),
    };

    let head_repo = ctx
        .db
        .find_repository_by_id(&row.head_repo_id)
        .await
        .map_err(db_err)?
        .ok_or_else(|| AppError::new("pull.internal", "head repository missing"))?;
    let (head_owner, head_name) = match head_repo.owner_type.as_str() {
        "org" => {
            let org = ctx
                .db
                .find_organization_by_id(&head_repo.owner_id)
                .await
                .map_err(db_err)?;
            (
                org.map(|o| o.slug).unwrap_or_default(),
                head_repo.name.clone(),
            )
        }
        _ => {
            let user = ctx
                .db
                .find_user_by_id(&head_repo.owner_id)
                .await
                .map_err(db_err)?;
            (
                user.map(|u| u.username).unwrap_or_default(),
                head_repo.name.clone(),
            )
        }
    };

    Ok(PullPublic {
        id: row.id.clone(),
        repo_id: row.repo_id.clone(),
        number: row.number,
        title: row.title.clone(),
        body: row.body.clone(),
        state,
        draft: row.draft,
        author_id: row.author_id.clone(),
        author_username,
        base_ref: row.base_ref.clone(),
        base_sha: row.base_sha.clone(),
        head_repo_id: row.head_repo_id.clone(),
        head_owner,
        head_name,
        head_ref: row.head_ref.clone(),
        head_sha: row.head_sha.clone(),
        merged_at: row.merged_at.clone(),
        merged_by: row.merged_by.clone(),
        merge_commit_sha: row.merge_commit_sha.clone(),
        merge_method,
        closed_at: row.closed_at.clone(),
        closed_by: row.closed_by.clone(),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    })
}

async fn load_pull_in_repo(
    ctx: &RpcCtx,
    repo_id: &str,
    number: i64,
) -> Result<PullRow, AppError> {
    if number < 1 {
        return Err(pull_not_found());
    }
    ctx.db
        .find_pull_by_repo_number(repo_id, number)
        .await
        .map_err(db_err)?
        .ok_or_else(pull_not_found)
}

/// `pull.create` — Write+; shared `#N` with issues (D-PR-02).
pub async fn create(ctx: &RpcCtx, input: serde_json::Value) -> Result<PullPublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: CreatePullRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid pull.create input: {e}"),
        )
    })?;
    let title = validate_title(&req.title)?.to_string();
    let body = validate_body(req.body.as_deref())?;
    let base_ref = validate_ref_name(&req.base_ref, "base_ref")?;
    let head_ref = validate_ref_name(&req.head_ref, "head_ref")?;
    let draft = req.draft.unwrap_or(false);

    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;

    let head_owner = req
        .head_owner
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(accessible.owner_username.as_str())
        .to_string();
    let head_name = req
        .head_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(accessible.row.name.as_str())
        .to_string();

    let (head_repo_id, head_owner_slug, head_repo_name) =
        if head_owner == accessible.owner_username && head_name == accessible.row.name {
            (
                accessible.row.id.clone(),
                accessible.owner_username.clone(),
                accessible.row.name.clone(),
            )
        } else {
            let head = crate::repo::resolve_repo_for_read(ctx, &head_owner, &head_name).await?;
            let network = ctx
                .db
                .get_repo_fork_network_id(&head.row.id)
                .await
                .map_err(db_err)?;
            if !crate::repo::head_valid_for_base(
                &accessible.row.id,
                &head.row.id,
                network.as_deref(),
            ) {
                return Err(AppError::new(
                    "pull.invalid_head",
                    "head repository must be this repo or a fork of it",
                ));
            }
            (head.row.id.clone(), head.owner_username.clone(), head.row.name.clone())
        };

    let base_sha =
        resolve_ref_sha(ctx, &accessible.owner_username, &accessible.row.name, &base_ref).await?;
    let head_sha = resolve_ref_sha(ctx, &head_owner_slug, &head_repo_name, &head_ref).await?;

    if base_ref == head_ref && accessible.row.id == head_repo_id && base_sha == head_sha {
        return Err(AppError::new(
            "rpc.bad_input",
            "base and head refs are identical",
        ));
    }

    let number = ctx
        .db
        .allocate_next_issue_number(&accessible.row.id)
        .await
        .map_err(db_err)?;
    let id = Uuid::new_v4().to_string();
    let row = ctx
        .db
        .insert_pull(
            &id,
            &accessible.row.id,
            number,
            &title,
            &body,
            &user.id,
            &base_ref,
            &base_sha,
            &head_repo_id,
            &head_ref,
            &head_sha,
            draft,
        )
        .await
        .map_err(db_err)?;
    let subject = notify::subject_for_pull(&row);
    let mentions = notify::resolve_mention_user_ids(ctx, &body).await;
    let requested = ctx
        .db
        .list_pull_review_request_user_ids(&row.id)
        .await
        .unwrap_or_default();
    let mut recipients = requested;
    for m in &mentions {
        if !recipients.iter().any(|r| r == m) {
            recipients.push(m.clone());
        }
    }
    notify::fanout(ctx, &user.id, recipients.clone(), "pr_opened", &subject).await;
    notify::fanout(ctx, &user.id, mentions, "pr_mention", &subject).await;
    emit_pull_event(
        ctx,
        &accessible,
        &row,
        "opened",
        &user.username,
        &user.id,
        false,
    )
    .await;
    to_public(ctx, &row).await
}

/// `pull.get` — Read+.
pub async fn get(ctx: &RpcCtx, input: serde_json::Value) -> Result<PullPublic, AppError> {
    let req: PullRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid pull.get input: {e}"))
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let row = load_pull_in_repo(ctx, &accessible.row.id, req.number).await?;
    to_public(ctx, &row).await
}

/// `pull.list` — Read+; default state `open` (D-PR-26).
pub async fn list(ctx: &RpcCtx, input: serde_json::Value) -> Result<PullListResponse, AppError> {
    let req: PullListRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid pull.list input: {e}"))
    })?;
    let accessible = acl::resolve_for_read(ctx, &req.owner, &req.name).await?;
    let state = req.state.as_deref().unwrap_or("open");
    let state_filter = match state {
        "all" => None,
        "closed" => Some("closed"), // note: merged listed separately via state=merged or all
        "merged" => Some("merged"),
        "open" => Some("open"),
        other => {
            return Err(AppError::new(
                "rpc.bad_input",
                format!("invalid pull state filter: {other}"),
            ))
        }
    };
    let offset = req.offset.unwrap_or(0);
    let limit = req.limit.unwrap_or(25);

    let author_id = resolve_username_filter(ctx, req.author.as_deref()).await?;
    if req
        .author
        .as_deref()
        .map(str::trim)
        .is_some_and(|s| !s.is_empty())
        && author_id.is_none()
    {
        return Ok(PullListResponse {
            pulls: vec![],
            total: 0,
            offset,
            limit,
        });
    }
    let assignee_id = resolve_username_filter(ctx, req.assignee.as_deref()).await?;
    if req
        .assignee
        .as_deref()
        .map(str::trim)
        .is_some_and(|s| !s.is_empty())
        && assignee_id.is_none()
    {
        return Ok(PullListResponse {
            pulls: vec![],
            total: 0,
            offset,
            limit,
        });
    }
    let label_filter = req
        .label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let review_state = req
        .review_state
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let needs_post_filter = label_filter.is_some()
        || assignee_id.is_some()
        || review_state.is_some()
        || state == "closed"
        || (state == "merged" && author_id.is_some());

    let (rows, total) = if needs_post_filter || author_id.is_some() {
        // Author via DB when possible; closed/label/assignee/review post-filtered (D-PR-26).
        let fetch_limit = if needs_post_filter { 500 } else { limit };
        let fetch_offset = if needs_post_filter { 0 } else { offset };
        let search_state = if state == "closed" || state == "merged" {
            "all"
        } else if state == "all" {
            "all"
        } else {
            state
        };
        let (all, _) = ctx
            .db
            .search_pulls_for_repo(
                &accessible.row.id,
                octanest_db::PullSearchFilters {
                    state: search_state,
                    author_id: author_id.as_deref(),
                    q: None,
                    offset: fetch_offset,
                    limit: fetch_limit,
                },
            )
            .await
            .map_err(db_err)?;
        let mut filtered = all;
        if state == "closed" {
            filtered.retain(|r| r.state == "closed" || r.state == "merged");
        }
        if state == "merged" {
            filtered.retain(|r| r.state == "merged");
        }
        if let Some(ref lid) = label_filter {
            let mut keep = Vec::new();
            for row in filtered {
                if pull_matches_label(ctx, &row.id, lid).await? {
                    keep.push(row);
                }
            }
            filtered = keep;
        }
        if let Some(ref aid) = assignee_id {
            let mut keep = Vec::new();
            for row in filtered {
                if pull_matches_assignee(ctx, &row.id, aid).await? {
                    keep.push(row);
                }
            }
            filtered = keep;
        }
        if let Some(ref rs) = review_state {
            let mut keep = Vec::new();
            for row in filtered {
                if pull_matches_review_state(ctx, &row.id, rs).await? {
                    keep.push(row);
                }
            }
            filtered = keep;
        }
        let total = filtered.len() as i64;
        let page: Vec<_> = if needs_post_filter {
            filtered
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect()
        } else {
            filtered
        };
        (page, total)
    } else {
        ctx.db
            .list_pulls_for_repo(&accessible.row.id, state_filter, offset, limit)
            .await
            .map_err(db_err)?
    };

    let mut pulls = Vec::with_capacity(rows.len());
    for row in rows {
        pulls.push(to_public(ctx, &row).await?);
    }
    Ok(PullListResponse {
        pulls,
        total,
        offset,
        limit,
    })
}

async fn resolve_username_filter(
    ctx: &RpcCtx,
    raw: Option<&str>,
) -> Result<Option<String>, AppError> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let username = raw.strip_prefix('@').unwrap_or(raw);
    let user = ctx
        .db
        .find_user_by_username(username)
        .await
        .map_err(db_err)?;
    Ok(user.map(|u| u.id))
}

async fn pull_matches_label(ctx: &RpcCtx, pull_id: &str, label: &str) -> Result<bool, AppError> {
    ctx.db
        .pull_has_label(pull_id, label)
        .await
        .map_err(db_err)
}

async fn pull_matches_assignee(
    ctx: &RpcCtx,
    pull_id: &str,
    user_id: &str,
) -> Result<bool, AppError> {
    ctx.db
        .pull_has_assignee(pull_id, user_id)
        .await
        .map_err(db_err)
}

async fn pull_matches_review_state(
    ctx: &RpcCtx,
    pull_id: &str,
    want: &str,
) -> Result<bool, AppError> {
    let reviews = ctx.db.list_pull_reviews(pull_id).await.map_err(db_err)?;
    let mut latest: std::collections::BTreeMap<String, &octanest_db::PullReviewRow> =
        std::collections::BTreeMap::new();
    for r in &reviews {
        match latest.get(&r.author_id) {
            Some(prev) if prev.submitted_at >= r.submitted_at => {}
            _ => {
                latest.insert(r.author_id.clone(), r);
            }
        }
    }
    let has_changes = latest.values().any(|r| r.state == "changes_requested");
    let has_approved = latest.values().any(|r| r.state == "approved");
    let actual = if has_changes {
        "changes_requested"
    } else if has_approved {
        "approved"
    } else {
        "review_required"
    };
    Ok(actual == want)
}

/// `pull.close` — Write+; open → closed (PR-06).
pub async fn close(ctx: &RpcCtx, input: serde_json::Value) -> Result<PullPublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: PullRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid pull.close input: {e}"))
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let row = load_pull_in_repo(ctx, &accessible.row.id, req.number).await?;
    if row.state != "open" {
        return Err(AppError::new(
            "pull.invalid_state",
            "only open pull requests can be closed",
        ));
    }
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    ctx.db
        .set_pull_state(&row.id, "closed", Some(&now), Some(&user.id))
        .await
        .map_err(db_err)?;
    let updated = load_pull_in_repo(ctx, &accessible.row.id, req.number).await?;
    let subject = notify::subject_for_pull(&updated);
    let recipients = notify::pull_participant_ids(ctx, &updated.id, &updated.author_id).await;
    notify::fanout(ctx, &user.id, recipients, "pr_closed", &subject).await;
    emit_pull_event(
        ctx,
        &accessible,
        &updated,
        "closed",
        &user.username,
        &user.id,
        false,
    )
    .await;
    to_public(ctx, &updated).await
}

/// `pull.reopen` — Write+; closed → open (not merged) (PR-06).
pub async fn reopen(ctx: &RpcCtx, input: serde_json::Value) -> Result<PullPublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: PullRefRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid pull.reopen input: {e}"))
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let row = load_pull_in_repo(ctx, &accessible.row.id, req.number).await?;
    if row.state != "closed" {
        return Err(AppError::new(
            "pull.invalid_state",
            "only closed (unmerged) pull requests can be reopened",
        ));
    }
    ctx.db
        .set_pull_state(&row.id, "open", None, None)
        .await
        .map_err(db_err)?;
    let updated = load_pull_in_repo(ctx, &accessible.row.id, req.number).await?;
    let subject = notify::subject_for_pull(&updated);
    let recipients = notify::pull_participant_ids(ctx, &updated.id, &updated.author_id).await;
    notify::fanout(ctx, &user.id, recipients, "pr_reopened", &subject).await;
    emit_pull_event(
        ctx,
        &accessible,
        &updated,
        "reopened",
        &user.username,
        &user.id,
        false,
    )
    .await;
    to_public(ctx, &updated).await
}

/// `pull.update` — Write+; title/body/base_ref/draft (D-PR-04 partial).
pub async fn update(ctx: &RpcCtx, input: serde_json::Value) -> Result<PullPublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: UpdatePullRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid pull.update input: {e}"),
        )
    })?;
    let accessible = acl::resolve_for_write(ctx, &req.owner, &req.name).await?;
    let row = load_pull_in_repo(ctx, &accessible.row.id, req.number).await?;
    if row.state == "merged" {
        return Err(AppError::new(
            "pull.invalid_state",
            "merged pull requests cannot be updated",
        ));
    }
    let title = match req.title.as_deref() {
        Some(t) => validate_title(t)?.to_string(),
        None => row.title.clone(),
    };
    let body = match req.body.as_deref() {
        Some(b) => validate_body(Some(b))?,
        None => row.body.clone(),
    };
    let draft = req.draft.unwrap_or(row.draft);
    let (base_ref, base_sha) = if let Some(br) = req.base_ref.as_deref() {
        let base_ref = validate_ref_name(br, "base_ref")?;
        let base_sha =
            resolve_ref_sha(ctx, &accessible.owner_username, &accessible.row.name, &base_ref)
                .await?;
        (base_ref, base_sha)
    } else {
        (row.base_ref.clone(), row.base_sha.clone())
    };

    let new_head_sha = if row.head_repo_id == row.repo_id {
        resolve_ref_sha(
            ctx,
            &accessible.owner_username,
            &accessible.row.name,
            &row.head_ref,
        )
        .await
        .unwrap_or_else(|_| row.head_sha.clone())
    } else {
        row.head_sha.clone()
    };
    let head_changed = new_head_sha != row.head_sha;
    let base_changed = base_ref != row.base_ref || base_sha != row.base_sha;
    let content_changed = title != row.title || body != row.body || draft != row.draft;

    ctx.db
        .update_pull_fields(
            &row.id,
            &title,
            &body,
            draft,
            &base_ref,
            &base_sha,
        )
        .await
        .map_err(db_err)?;
    if head_changed {
        ctx.db
            .update_pull_head_sha(&row.id, &new_head_sha)
            .await
            .map_err(db_err)?;
    }
    if head_changed || base_changed {
        ctx.db
            .mark_pull_line_comments_outdated(&row.id)
            .await
            .map_err(db_err)?;
    }
    let updated = load_pull_in_repo(ctx, &accessible.row.id, req.number).await?;
    let action = if head_changed {
        "synchronize"
    } else if content_changed || base_changed {
        "edited"
    } else {
        "edited"
    };
    emit_pull_event(
        ctx,
        &accessible,
        &updated,
        action,
        &user.username,
        &user.id,
        false,
    )
    .await;
    to_public(ctx, &updated).await
}
