//! Best-effort notification fan-out after domain writes (NOTF-01 / D-03).

use std::collections::HashSet;

use uuid::Uuid;

use crate::rpc::RpcCtx;

/// Subject metadata for a notification row.
#[derive(Debug, Clone)]
pub struct NotifySubject {
    pub kind: &'static str,
    pub repo_id: String,
    pub number: i64,
    pub title: String,
}

/// Insert one notification per recipient, excluding the actor (D-03).
/// Soft-fails: logs errors and does not abort the caller.
pub async fn fanout(
    ctx: &RpcCtx,
    actor_id: &str,
    recipients: impl IntoIterator<Item = String>,
    reason: &str,
    subject: &NotifySubject,
) {
    let mut seen = HashSet::new();
    for recipient_id in recipients {
        if recipient_id.is_empty() || recipient_id == actor_id {
            continue;
        }
        if !seen.insert(recipient_id.clone()) {
            continue;
        }
        let id = Uuid::new_v4().to_string();
        if let Err(e) = ctx
            .db
            .insert_notification(
                &id,
                &recipient_id,
                actor_id,
                reason,
                subject.kind,
                &subject.repo_id,
                subject.number,
                &subject.title,
            )
            .await
        {
            tracing::warn!(
                error = %e,
                recipient_id = %recipient_id,
                reason = %reason,
                "notification fanout insert failed (soft-fail)"
            );
        }
    }
}

/// Extract `@username` handles from plain text.
pub fn extract_mention_usernames(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() {
                let c = bytes[end] as char;
                if c.is_ascii_alphanumeric() || c == '-' {
                    end += 1;
                    if end - start > 39 {
                        break;
                    }
                } else {
                    break;
                }
            }
            if end > start && end - start <= 39 {
                let prev_ok = i == 0
                    || matches!(
                        bytes[i - 1] as char,
                        ' ' | '\t' | '\n' | '\r' | '(' | '[' | '{' | ',' | ':' | ';'
                    );
                if prev_ok {
                    if let Ok(name) = std::str::from_utf8(&bytes[start..end]) {
                        let lower = name.to_ascii_lowercase();
                        if seen.insert(lower.clone()) {
                            out.push(lower);
                        }
                    }
                }
            }
            i = end.max(i + 1);
            continue;
        }
        i += 1;
    }
    out
}

/// Resolve `@username` mentions to user ids (unknown handles ignored).
pub async fn resolve_mention_user_ids(ctx: &RpcCtx, body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for username in extract_mention_usernames(body) {
        match ctx.db.find_user_by_username(&username).await {
            Ok(Some(u)) => ids.push(u.id),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, username = %username, "mention resolve failed");
            }
        }
    }
    ids
}

/// Issue participants: author + assignees + prior commenters (D-03).
pub async fn issue_participant_ids(ctx: &RpcCtx, issue_id: &str, author_id: &str) -> Vec<String> {
    let mut ids = HashSet::new();
    if !author_id.is_empty() {
        ids.insert(author_id.to_string());
    }
    match ctx.db.list_issue_assignees(issue_id).await {
        Ok(rows) => {
            for a in rows {
                ids.insert(a.user_id);
            }
        }
        Err(e) => tracing::warn!(error = %e, "list_issue_assignees for notify failed"),
    }
    match ctx.db.list_issue_comments(issue_id).await {
        Ok(rows) => {
            for c in rows {
                ids.insert(c.author_id);
            }
        }
        Err(e) => tracing::warn!(error = %e, "list_issue_comments for notify failed"),
    }
    ids.into_iter().collect()
}

pub fn subject_for_issue(issue: &oxidean_db::IssueRow) -> NotifySubject {
    NotifySubject {
        kind: "issue",
        repo_id: issue.repo_id.clone(),
        number: issue.number,
        title: issue.title.clone(),
    }
}

pub fn subject_for_pull(pull: &oxidean_db::PullRow) -> NotifySubject {
    NotifySubject {
        kind: "pull_request",
        repo_id: pull.repo_id.clone(),
        number: pull.number,
        title: pull.title.clone(),
    }
}

/// PR participants: author + prior commenters + requested reviewers (D-02 / D-03).
pub async fn pull_participant_ids(ctx: &RpcCtx, pull_id: &str, author_id: &str) -> Vec<String> {
    let mut ids = HashSet::new();
    if !author_id.is_empty() {
        ids.insert(author_id.to_string());
    }
    match ctx.db.list_pull_comments(pull_id).await {
        Ok(rows) => {
            for c in rows {
                ids.insert(c.author_id);
            }
        }
        Err(e) => tracing::warn!(error = %e, "list_pull_comments for notify failed"),
    }
    match ctx.db.list_pull_review_request_user_ids(pull_id).await {
        Ok(rows) => {
            for uid in rows {
                ids.insert(uid);
            }
        }
        Err(e) => tracing::warn!(error = %e, "list_pull_review_requests for notify failed"),
    }
    match ctx.db.list_pull_reviews(pull_id).await {
        Ok(rows) => {
            for r in rows {
                ids.insert(r.author_id);
            }
        }
        Err(e) => tracing::warn!(error = %e, "list_pull_reviews for notify failed"),
    }
    ids.into_iter().collect()
}
