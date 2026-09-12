//! Owner-only private ACL stub + unified not_found (D-23–D-25 / T-07-13).

use octanest_core::AppError;
use octanest_db::RepositoryRow;

use crate::rpc::RpcCtx;

/// Identical error for missing repos and unauthorized private access (anti-enumeration).
pub fn not_found() -> AppError {
    AppError::new("repo.not_found", "Repository not found")
}

/// Repo row the caller is allowed to read, plus resolved owner username.
pub struct AccessibleRepo {
    pub row: RepositoryRow,
    pub owner_username: String,
}

/// Resolve `owner`/`name` for read. Missing OR private and caller ≠ owner → identical [`not_found`].
pub async fn resolve_repo_for_read(
    ctx: &RpcCtx,
    owner: &str,
    name: &str,
) -> Result<AccessibleRepo, AppError> {
    let owner = owner.trim();
    let name = name.trim();
    if owner.is_empty() || name.is_empty() {
        return Err(not_found());
    }

    let owner_user = match ctx.db.find_user_by_username(owner).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(not_found()),
        Err(e) => {
            tracing::error!(error = %e, "find_user_by_username failed");
            return Err(AppError::new("repo.internal", "repository operation failed"));
        }
    };

    let row = match ctx
        .db
        .find_repository_by_owner_name(&owner_user.id, name)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return Err(not_found()),
        Err(e) => {
            tracing::error!(error = %e, "find_repository_by_owner_name failed");
            return Err(AppError::new("repo.internal", "repository operation failed"));
        }
    };

    let is_private = row.visibility.eq_ignore_ascii_case("private");
    if is_private {
        let caller_id = ctx.session.as_ref().map(|s| s.user_id.as_str());
        if caller_id != Some(owner_user.id.as_str()) {
            return Err(not_found());
        }
    }

    Ok(AccessibleRepo {
        row,
        owner_username: owner_user.username,
    })
}
