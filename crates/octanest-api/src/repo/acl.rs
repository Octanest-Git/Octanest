//! Owner-only private ACL stub + unified not_found (D-23–D-25 / T-07-13).
//!
//! Shared *decision* helpers are reused by Smart HTTP; web vs git *status mapping*
//! stays separate (web → `repo.not_found`; git private unauth → 401, D-21).

use octanest_core::AppError;
use octanest_db::{Database, RepositoryRow};

use crate::rpc::RpcCtx;

/// Identical error for missing repos and unauthorized private access (anti-enumeration).
pub fn not_found() -> AppError {
    AppError::new("repo.not_found", "Repository not found")
}

/// Whether visibility is private (case-insensitive).
pub fn is_private_visibility(visibility: &str) -> bool {
    visibility.eq_ignore_ascii_case("private")
}

/// Owner-only private read until Phase 10 collaborators / plan 04 ACL rewrite.
pub fn can_read_as_owner(caller_user_id: Option<&str>, owner_id: &str) -> bool {
    caller_user_id == Some(owner_id)
}

/// Polymorphic owner resolved from a shared slug (D-ORG-01 / RESEARCH Pattern 3).
#[derive(Debug, Clone)]
pub enum OwnerRef {
    User { id: String, username: String },
    Org { id: String, slug: String },
}

impl OwnerRef {
    pub fn id(&self) -> &str {
        match self {
            Self::User { id, .. } | Self::Org { id, .. } => id,
        }
    }

    /// Public URL / disk path segment (username or org slug).
    pub fn slug(&self) -> &str {
        match self {
            Self::User { username, .. } => username,
            Self::Org { slug, .. } => slug,
        }
    }

    pub fn owner_type(&self) -> &'static str {
        match self {
            Self::User { .. } => "user",
            Self::Org { .. } => "org",
        }
    }
}

/// Resolve shared slug → user (first) or organization. Missing → `Ok(None)`.
pub async fn resolve_owner_slug(
    db: &Database,
    slug: &str,
) -> Result<Option<OwnerRef>, String> {
    let slug = slug.trim();
    if slug.is_empty() {
        return Ok(None);
    }
    if let Some(u) = db.find_user_by_username(slug).await? {
        return Ok(Some(OwnerRef::User {
            id: u.id,
            username: u.username,
        }));
    }
    if let Some(o) = db.find_organization_by_slug(slug).await? {
        return Ok(Some(OwnerRef::Org {
            id: o.id,
            slug: o.slug,
        }));
    }
    Ok(None)
}

/// Repo row the caller is allowed to read, plus resolved owner username/slug.
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

    let owner_ref = match resolve_owner_slug(&ctx.db, owner).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(not_found()),
        Err(e) => {
            tracing::error!(error = %e, "resolve_owner_slug failed");
            return Err(AppError::new("repo.internal", "repository operation failed"));
        }
    };

    let row = match ctx
        .db
        .find_repository_by_owner_name(owner_ref.id(), name)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return Err(not_found()),
        Err(e) => {
            tracing::error!(error = %e, "find_repository_by_owner_name failed");
            return Err(AppError::new("repo.internal", "repository operation failed"));
        }
    };

    if is_private_visibility(&row.visibility) {
        let caller_id = ctx.session.as_ref().map(|s| s.user_id.as_str());
        // Temporary: personal owner_id match only (plan 04 rewrites ACL for org roles).
        if !can_read_as_owner(caller_id, owner_ref.id()) {
            return Err(not_found());
        }
    }

    Ok(AccessibleRepo {
        row,
        owner_username: owner_ref.slug().to_string(),
    })
}

#[cfg(test)]
mod coalesce_stubs {
    //! Wave 0 / ORG-02 / D-ORG-05: highest-wins coalesce matrix stubs.
    //! RED until Capability + coalesce land — do not change production can_read_as_owner yet.

    /// Personal owner → admin (D-ORG-05).
    #[test]
    fn coalesce_personal_owner_is_admin() {
        assert!(
            false,
            "Wave 0: personal owner coalesce → Capability::Admin (ORG-02 / D-ORG-05)"
        );
    }

    /// Org Owner → admin regardless of member_base.
    #[test]
    fn coalesce_org_owner_is_admin() {
        assert!(
            false,
            "Wave 0: org Owner coalesce → Admin (ORG-02 / D-ORG-02a)"
        );
    }

    /// Org Admin → admin regardless of member_base.
    #[test]
    fn coalesce_org_admin_is_admin() {
        assert!(
            false,
            "Wave 0: org Admin coalesce → Admin (ORG-02 / D-ORG-02a)"
        );
    }

    /// Member × member_base=none → no capability from org role alone.
    #[test]
    fn coalesce_member_base_none_yields_none() {
        assert!(
            false,
            "Wave 0: Member + member_base none → no org capability (ORG-02 / D-ORG-02b)"
        );
    }

    /// Member × member_base=read → Read.
    #[test]
    fn coalesce_member_base_read_is_read() {
        assert!(
            false,
            "Wave 0: Member + member_base read → Read (ORG-02 / D-ORG-02b)"
        );
    }

    /// Member × member_base=write → Write.
    #[test]
    fn coalesce_member_base_write_is_write() {
        assert!(
            false,
            "Wave 0: Member + member_base write → Write (ORG-02 / D-ORG-02b)"
        );
    }

    /// Collaborator grant raises Member with base none.
    #[test]
    fn coalesce_collaborator_raises_member_base_none() {
        assert!(
            false,
            "Wave 0: Collaborator raise over Member base none (ORG-02/03 / D-ORG-05)"
        );
    }

    /// Public visibility grants at least Read.
    #[test]
    fn coalesce_public_repo_grants_read() {
        assert!(
            false,
            "Wave 0: public_repo bump → Read (ORG-02 / D-ORG-05)"
        );
    }

    /// Anonymous private → none (no grant sources).
    #[test]
    fn coalesce_anonymous_private_is_none() {
        assert!(
            false,
            "Wave 0: anonymous private coalesce → None (ORG-04 / D-ORG-05)"
        );
    }
}
