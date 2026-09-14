//! Issue ACL helpers — reuse repo Capability ladder (D-ISS-20).

use octanest_core::AppError;
use octanest_db::IssueRow;

use crate::repo::{
    meets, not_found, resolve_repo_for_admin, resolve_repo_for_read, AccessibleRepo, Capability,
};
use crate::rpc::RpcCtx;

/// Resolve repo for Read+ (list/get). Missing / unauthorized private → soft `repo.not_found`.
pub async fn resolve_for_read(
    ctx: &RpcCtx,
    owner: &str,
    name: &str,
) -> Result<AccessibleRepo, AppError> {
    resolve_repo_for_read(ctx, owner, name).await
}

/// Resolve repo for Write+ (create / close / reopen). Soft not_found when capability is insufficient.
pub async fn resolve_for_write(
    ctx: &RpcCtx,
    owner: &str,
    name: &str,
) -> Result<AccessibleRepo, AppError> {
    let accessible = resolve_repo_for_read(ctx, owner, name).await?;
    if !meets(accessible.capability, Capability::Write) {
        return Err(not_found());
    }
    Ok(accessible)
}

/// Resolve repo for Admin (hard-delete). Soft not_found when capability is insufficient.
pub async fn resolve_for_admin(
    ctx: &RpcCtx,
    owner: &str,
    name: &str,
) -> Result<AccessibleRepo, AppError> {
    resolve_repo_for_admin(ctx, owner, name).await
}

/// Author **or** Write+ may edit title/body (D-ISS-03 / D-ISS-20).
pub fn can_edit_issue(user_id: &str, issue: &IssueRow, capability: Option<Capability>) -> bool {
    issue.author_id == user_id || meets(capability, Capability::Write)
}
