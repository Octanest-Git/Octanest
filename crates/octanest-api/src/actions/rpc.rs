//! `repo.actions.*` session RPC (ACT-03 / D-ACT-12 / D-ACT-13 / D-ACT-18).

use octanest_core::{
    ActionJobLogRequest, ActionJobLogResponse, ActionJobPublic, ActionRunGetRequest,
    ActionRunGetResponse, ActionRunPublic, ActionRunsListRequest, ActionRunsListResponse, AppError,
};
use octanest_db::{ActionJobRow, ActionRunRow};

use crate::actions::logs::read_job_log;
use crate::auth::gate::require_verified;
use crate::repo::{meets, not_found, resolve_repo_for_read, Capability};
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
