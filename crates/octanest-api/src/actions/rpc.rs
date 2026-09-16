//! `repo.actions.*` session RPC (ACT-03 / D-ACT-12 / D-ACT-13 / D-ACT-18 / ACT-06).

use uuid::Uuid;

use octanest_core::{
    ActionEnabledRequest, ActionEnabledResponse, ActionJobLogRequest, ActionJobLogResponse,
    ActionJobPublic, ActionListRunnersResponse, ActionRegistrationTokenResponse,
    ActionRunGetRequest, ActionRunGetResponse, ActionRunPublic, ActionRunnerPublic,
    ActionRunsListRequest, ActionRunsListResponse, ActionSecretDeleteRequest,
    ActionSecretMetaPublic, ActionSecretPutRequest, ActionSecretsListRequest,
    ActionSecretsListResponse, ActionSetEnabledRequest, AppError,
};
use octanest_db::{ActionJobRow, ActionRunRow, ActionRunnerRow};

use crate::actions::logs::read_job_log;
use crate::actions::secrets::{encrypt_secret, validate_secret_name};
use crate::actions::tokens::mint_registration_token;
use crate::auth::admin::require_admin as require_sys_admin;
use crate::auth::gate::require_verified;
use crate::repo::{
    meets, not_found, resolve_repo_for_admin, resolve_repo_for_read, Capability,
};
use crate::rpc::RpcCtx;

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new(
            "db.not_configured",
            "no database configured for this instance",
        )
    } else {
        tracing::error!(error = %e, "actions rpc db error");
        AppError::new("repo.internal", "repository operation failed")
    }
}

fn run_public(r: &ActionRunRow) -> ActionRunPublic {
    ActionRunPublic {
        id: r.id.clone(),
        repository_id: r.repository_id.clone(),
        workflow_path: r.workflow_path.clone(),
        workflow_name: r.workflow_name.clone(),
        event: r.event.clone(),
        head_sha: r.head_sha.clone(),
        head_ref: r.head_ref.clone(),
        status: r.status.clone(),
        title: r.title.clone(),
    }
}

fn job_public(j: &ActionJobRow) -> ActionJobPublic {
    let runs_on: Vec<String> = serde_json::from_str(&j.runs_on_json).unwrap_or_default();
    ActionJobPublic {
        id: j.id.clone(),
        run_id: j.run_id.clone(),
        job_key: j.job_key.clone(),
        name: j.name.clone(),
        status: j.status.clone(),
        runs_on,
    }
}

fn runner_public(r: &ActionRunnerRow) -> ActionRunnerPublic {
    let labels: Vec<String> = serde_json::from_str(&r.labels_json).unwrap_or_default();
    ActionRunnerPublic {
        id: r.id.clone(),
        name: r.name.clone(),
        labels,
        repository_id: r.repository_id.clone(),
        ephemeral: r.ephemeral,
        last_online: r.last_online.clone(),
        created_at: r.created_at.clone(),
    }
}

/// `repo.actions.listRuns` — Read+.
pub async fn list_runs(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<ActionRunsListResponse, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionRunsListRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.listRuns input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    if !meets(accessible.capability, Capability::Read) {
        return Err(not_found());
    }
    let rows = ctx
        .db
        .list_action_runs_for_repo(&accessible.row.id)
        .await
        .map_err(db_err)?;
    Ok(ActionRunsListResponse {
        runs: rows.iter().map(run_public).collect(),
    })
}

/// `repo.actions.getRun` — Read+.
pub async fn get_run(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<ActionRunGetResponse, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionRunGetRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.getRun input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    if !meets(accessible.capability, Capability::Read) {
        return Err(not_found());
    }
    let run = ctx
        .db
        .find_action_run_by_id(&req.run_id)
        .await
        .map_err(db_err)?
        .ok_or_else(not_found)?;
    if run.repository_id != accessible.row.id {
        return Err(not_found());
    }
    let jobs = ctx
        .db
        .list_action_jobs_for_run(&run.id)
        .await
        .map_err(db_err)?;
    Ok(ActionRunGetResponse {
        run: run_public(&run),
        jobs: jobs.iter().map(job_public).collect(),
    })
}

/// `repo.actions.getJobLog` — Read+.
pub async fn get_job_log(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<ActionJobLogResponse, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionJobLogRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.getJobLog input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    if !meets(accessible.capability, Capability::Read) {
        return Err(not_found());
    }
    let run = ctx
        .db
        .find_action_run_by_id(&req.run_id)
        .await
        .map_err(db_err)?
        .ok_or_else(not_found)?;
    if run.repository_id != accessible.row.id {
        return Err(not_found());
    }
    let job = ctx
        .db
        .find_action_job_by_id(&req.job_id)
        .await
        .map_err(db_err)?
        .ok_or_else(not_found)?;
    if job.run_id != run.id {
        return Err(not_found());
    }
    let bytes = read_job_log(&ctx.actions_log_dir, &req.run_id, &req.job_id)
        .await
        .unwrap_or_default();
    Ok(ActionJobLogResponse {
        content: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

/// `repo.actions.secrets.list` — Admin; names only (D-ACT-17).
pub async fn list_secrets(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<ActionSecretsListResponse, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionSecretsListRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.secrets.list input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let rows = ctx
        .db
        .list_action_secret_names(&accessible.row.id)
        .await
        .map_err(db_err)?;
    Ok(ActionSecretsListResponse {
        secrets: rows
            .into_iter()
            .map(|r| ActionSecretMetaPublic {
                name: r.name,
                updated_at: r.updated_at,
            })
            .collect(),
    })
}

/// `repo.actions.secrets.put` — Admin; value never returned on read (D-ACT-17).
pub async fn put_secret(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionSecretPutRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.secrets.put input: {e}"),
        )
    })?;
    validate_secret_name(&req.secret_name)
        .map_err(|msg| AppError::new("repo.actions.invalid_secret_name", msg))?;
    if req.value.is_empty() {
        return Err(AppError::new(
            "repo.actions.invalid_secret_value",
            "secret value is required",
        ));
    }
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let ciphertext = encrypt_secret(&req.value)
        .map_err(|e| AppError::new("repo.actions.secret_encrypt_failed", e))?;
    let _ = ctx
        .db
        .delete_action_secret_by_name(&accessible.row.id, &req.secret_name)
        .await
        .map_err(db_err)?;
    ctx.db
        .insert_action_secret(
            &Uuid::new_v4().to_string(),
            &accessible.row.id,
            &req.secret_name,
            &ciphertext,
        )
        .await
        .map_err(db_err)?;
    Ok(serde_json::json!({ "ok": true, "name": req.secret_name }))
}

/// `repo.actions.secrets.delete` — Admin.
pub async fn delete_secret(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionSecretDeleteRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.secrets.delete input: {e}"),
        )
    })?;
    validate_secret_name(&req.secret_name)
        .map_err(|msg| AppError::new("repo.actions.invalid_secret_name", msg))?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    let ok = ctx
        .db
        .delete_action_secret_by_name(&accessible.row.id, &req.secret_name)
        .await
        .map_err(db_err)?;
    Ok(serde_json::json!({ "ok": ok }))
}

/// `repo.actions.getEnabled` — Read+.
pub async fn get_enabled(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<ActionEnabledResponse, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionEnabledRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.getEnabled input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_read(ctx, &req.owner, &req.name).await?;
    if !meets(accessible.capability, Capability::Read) {
        return Err(not_found());
    }
    let enabled = ctx
        .db
        .get_repo_actions_enabled(&accessible.row.id)
        .await
        .map_err(db_err)?;
    Ok(ActionEnabledResponse { enabled })
}

/// `repo.actions.setEnabled` — Admin (D-ACT-06).
pub async fn set_enabled(
    ctx: &RpcCtx,
    input: serde_json::Value,
) -> Result<ActionEnabledResponse, AppError> {
    let _ = require_verified(ctx).await?;
    let req: ActionSetEnabledRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid repo.actions.setEnabled input: {e}"),
        )
    })?;
    let accessible = resolve_repo_for_admin(ctx, &req.owner, &req.name).await?;
    ctx.db
        .set_repo_actions_enabled(&accessible.row.id, req.enabled)
        .await
        .map_err(db_err)?;
    Ok(ActionEnabledResponse {
        enabled: req.enabled,
    })
}

/// `admin.actions.createRegistrationToken` — instance Admin; one-time token (D-ACT-08).
pub async fn admin_create_registration_token(
    ctx: &RpcCtx,
    _input: serde_json::Value,
) -> Result<ActionRegistrationTokenResponse, AppError> {
    require_sys_admin(ctx).await?;
    let token = mint_registration_token(&ctx.db)
        .await
        .map_err(|e| AppError::new("admin.actions.token_failed", e))?;
    Ok(ActionRegistrationTokenResponse {
        token,
        scope: "instance".into(),
    })
}

/// `admin.actions.listRunners` — instance Admin.
pub async fn admin_list_runners(
    ctx: &RpcCtx,
    _input: serde_json::Value,
) -> Result<ActionListRunnersResponse, AppError> {
    require_sys_admin(ctx).await?;
    let rows = ctx.db.list_action_runners().await.map_err(db_err)?;
    Ok(ActionListRunnersResponse {
        runners: rows.iter().map(runner_public).collect(),
    })
}
