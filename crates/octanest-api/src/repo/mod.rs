//! Repository RPC handlers (`repo.create` tracer — GIT-01 / D-11 / D-30–D-32).

use octanest_core::{
    validate_repo_name, AppError, CreateRepoRequest, RepoPublic, RepoVisibility,
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

/// `repo.create` — verified owner creates an empty public/private repo (DB + bare git).
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

    if ctx
        .db
        .find_repository_by_owner_name(&user.id, &name)
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
            &user.id,
            &name,
            map_visibility(visibility),
            &description,
            &default_branch,
        )
        .await
        .map_err(db_err)?;

    let path = bare_repo_path(&ctx.repos_dir, &user.username, &name)?;
    if let Err(e) = ctx.git.init_bare(&path, &default_branch).await {
        tracing::error!(
            error = %e,
            path = %path.display(),
            "init_bare failed after DB insert"
        );
        return Err(AppError::new(
            "repo.git_init_failed",
            "failed to initialize repository storage",
        ));
    }

    Ok(RepoPublic {
        id: row.id,
        owner_id: row.owner_id,
        owner_username: user.username,
        name: row.name,
        description: row.description,
        visibility,
        default_branch: row.default_branch,
        updated_at: row.updated_at,
    })
}
