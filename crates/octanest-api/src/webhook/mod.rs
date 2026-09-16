//! Admin-gated `webhook.*` RPC (HOOK-01 / HOOK-03, D-HOOK-02).

pub mod deliver;
pub mod dispatch;

use octanest_core::{
    AppError, CreateWebhookRequest, DeleteWebhookResponse, UpdateWebhookRequest, WebhookIdRequest,
    WebhookListRequest, WebhookListResponse, WebhookPublic,
};
use octanest_db::WebhookRow;
use uuid::Uuid;

use crate::auth::gate::require_verified;
use crate::repo::resolve_repo_for_admin;
use crate::rpc::RpcCtx;

use self::deliver::validate_webhook_url;

const ALLOWED_EVENTS: &[&str] = &["push", "pull_request", "issues", "ping", "*"];

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new("db.not_configured", "no database configured for this instance")
    } else if e == "webhook not found" {
        AppError::new("webhook.not_found", "Webhook not found")
    } else {
        tracing::error!("webhook db error: {e}");
        AppError::new("webhook.internal", "webhook operation failed")
    }
}

fn mask_secret(secret: &str) -> String {
    if secret.is_empty() {
        return "********".into();
    }
    let prefix: String = secret.chars().take(4).collect();
    format!("{prefix}********")
}

fn parse_events(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn to_public(row: &WebhookRow, reveal: Option<String>) -> WebhookPublic {
    WebhookPublic {
        id: row.id.clone(),
        repo_id: row.repository_id.clone(),
        url: row.url.clone(),
        secret_masked: mask_secret(&row.secret),
        active: row.active,
        events: parse_events(&row.events),
        name: row.name.clone(),
        created_by: row.created_by.clone(),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        secret: reveal,
    }
}

fn normalize_events(events: &[String]) -> Result<Vec<String>, AppError> {
    if events.is_empty() {
        return Err(AppError::new(
            "rpc.bad_input",
            "at least one event is required",
        ));
    }
    let mut out = Vec::new();
    for e in events {
        let e = e.trim();
        if e.is_empty() {
            continue;
        }
        if !ALLOWED_EVENTS.contains(&e) {
            return Err(AppError::new(
                "rpc.bad_input",
                format!("unsupported webhook event: {e}"),
            ));
        }
        if !out.iter().any(|x| x == e) {
            out.push(e.to_string());
        }
    }
    if out.is_empty() {
        return Err(AppError::new(
            "rpc.bad_input",
            "at least one event is required",
        ));
    }
    Ok(out)
}

async fn load_hook_in_repo(
    ctx: &RpcCtx,
    repo_id: &str,
    hook_id: &str,
) -> Result<WebhookRow, AppError> {
    let row = ctx.db.get_webhook(hook_id).await.map_err(db_err)?;
    if row.repository_id != repo_id {
        return Err(AppError::new("webhook.not_found", "Webhook not found"));
    }
    Ok(row)
}

pub async fn create(ctx: &RpcCtx, input: serde_json::Value) -> Result<WebhookPublic, AppError> {
    let user = require_verified(ctx).await?;
    let req: CreateWebhookRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid webhook.create input: {e}"))
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let url = req.url.trim();
    if url.is_empty() {
        return Err(AppError::new("rpc.bad_input", "url is required"));
    }
    validate_webhook_url(url, &ctx.env_name)
        .map_err(|e| AppError::new("webhook.invalid_url", e))?;
    let secret = req.secret.trim();
    if secret.is_empty() {
        return Err(AppError::new("rpc.bad_input", "secret is required"));
    }
    let events = normalize_events(&req.events)?;
    let events_json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".into());
    let active = req.active.unwrap_or(true);
    let name = req
        .description
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    let id = Uuid::new_v4().to_string();
    let row = ctx
        .db
        .insert_webhook(
            &id,
            &accessible.row.id,
            url,
            secret,
            active,
            &events_json,
            &name,
            &user.id,
        )
        .await
        .map_err(db_err)?;
    Ok(to_public(&row, Some(secret.to_string())))
}

pub async fn list(ctx: &RpcCtx, input: serde_json::Value) -> Result<WebhookListResponse, AppError> {
    let _user = require_verified(ctx).await?;
    let req: WebhookListRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid webhook.list input: {e}"))
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let rows = ctx
        .db
        .list_webhooks_for_repo(&accessible.row.id)
        .await
        .map_err(db_err)?;
    Ok(WebhookListResponse {
        webhooks: rows.iter().map(|r| to_public(r, None)).collect(),
    })
}

pub async fn get(ctx: &RpcCtx, input: serde_json::Value) -> Result<WebhookPublic, AppError> {
    let _user = require_verified(ctx).await?;
    let req: WebhookIdRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid webhook.get input: {e}"))
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let row = load_hook_in_repo(ctx, &accessible.row.id, &req.id).await?;
    Ok(to_public(&row, None))
}

pub async fn update(ctx: &RpcCtx, input: serde_json::Value) -> Result<WebhookPublic, AppError> {
    let _user = require_verified(ctx).await?;
    let req: UpdateWebhookRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid webhook.update input: {e}"))
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let _existing = load_hook_in_repo(ctx, &accessible.row.id, &req.id).await?;

    let url = req.url.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if let Some(u) = url {
        validate_webhook_url(u, &ctx.env_name)
            .map_err(|e| AppError::new("webhook.invalid_url", e))?;
    }
    let secret_reveal = req
        .secret
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let events_json = if let Some(events) = &req.events {
        Some(serde_json::to_string(&normalize_events(events)?).unwrap_or_else(|_| "[]".into()))
    } else {
        None
    };
    let name = req
        .description
        .as_deref()
        .map(|s| s.trim().to_string());

    let row = ctx
        .db
        .update_webhook(
            &req.id,
            url,
            secret_reveal.as_deref(),
            req.active,
            events_json.as_deref(),
            name.as_deref(),
        )
        .await
        .map_err(db_err)?;
    Ok(to_public(&row, secret_reveal))
}

pub async fn delete(ctx: &RpcCtx, input: serde_json::Value) -> Result<DeleteWebhookResponse, AppError> {
    let _user = require_verified(ctx).await?;
    let req: WebhookIdRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid webhook.delete input: {e}"))
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let _ = load_hook_in_repo(ctx, &accessible.row.id, &req.id).await?;
    ctx.db.delete_webhook(&req.id).await.map_err(db_err)?;
    Ok(DeleteWebhookResponse { ok: true })
}
