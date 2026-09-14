//! Organization RPC handlers (`org.create` — ORG-01 / D-ORG-01 / D-ORG-02a).

use octanest_core::{
    is_reserved_username, validate_username, AppError, CreateOrgRequest, MemberBasePermission,
    OrgPublic,
};
use uuid::Uuid;

use crate::auth::gate::require_verified;
use crate::rpc::RpcCtx;

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new(
            "db.not_configured",
            "no database configured for this instance",
        )
    } else if e.contains("UNIQUE") || e.contains("unique") || e.contains("Duplicate") {
        AppError::new(
            "org.slug_taken",
            "That slug is already used by a user or organization. Choose a different slug.",
        )
    } else {
        tracing::error!("org db error: {e}");
        AppError::new("org.internal", "organization operation failed")
    }
}

fn map_slug_err(msg: String) -> AppError {
    if msg.contains("reserved") {
        AppError::new("auth.reserved_username", "username is reserved")
    } else {
        AppError::new("auth.invalid_username", msg)
    }
}

fn to_public(row: &octanest_db::OrganizationRow) -> Result<OrgPublic, AppError> {
    let member_base_permission = MemberBasePermission::parse(&row.member_base_permission)
        .map_err(|e| {
            tracing::error!(error = %e, "invalid member_base_permission in org row");
            AppError::new("org.internal", "organization operation failed")
        })?;
    Ok(OrgPublic {
        id: row.id.clone(),
        slug: row.slug.clone(),
        display_name: row.display_name.clone(),
        member_base_permission,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    })
}

/// `org.create` — verified user creates an org and becomes Owner (ORG-01 / D-ORG-01 / D-ORG-02a).
pub async fn create(ctx: &RpcCtx, input: serde_json::Value) -> Result<OrgPublic, AppError> {
    let user = require_verified(ctx).await?;

    let req: CreateOrgRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new("rpc.bad_input", format!("invalid org.create input: {e}"))
    })?;

    validate_username(&req.slug).map_err(map_slug_err)?;
    let slug = req.slug.trim().to_string();

    if is_reserved_username(&slug) {
        return Err(AppError::new(
            "auth.reserved_username",
            "username is reserved",
        ));
    }

    let display_name = req
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(slug.as_str())
        .to_string();

    // D-ORG-01 / T-10-04: shared namespace — reject collisions with users or orgs.
    if ctx
        .db
        .find_user_by_username(&slug)
        .await
        .map_err(db_err)?
        .is_some()
        || ctx
            .db
            .find_organization_by_slug(&slug)
            .await
            .map_err(db_err)?
            .is_some()
    {
        return Err(AppError::new(
            "org.slug_taken",
            "That slug is already used by a user or organization. Choose a different slug.",
        ));
    }

    let id = Uuid::new_v4().to_string();
    let row = ctx
        .db
        .insert_organization(
            &id,
            &slug,
            &display_name,
            MemberBasePermission::None.as_str(),
        )
        .await
        .map_err(db_err)?;

    // Creator is sole Owner (ORG-01 / D-ORG-02a / T-10-05).
    ctx.db
        .insert_org_owner_membership(&row.id, &user.id)
        .await
        .map_err(db_err)?;

    to_public(&row)
}
