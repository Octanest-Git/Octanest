//! ORG-03 / ORG-04 Wave 0 stubs: repository collaborator CRUD + ACL matrix.
//!
//! RED until collaborator RPC + ACL coalesce land. Do not implement handlers here.
//! Threat: T-10-01 web private deny → repo.not_found; T-10-02 Member none + Collaborator raise.

/// Collaborator CRUD on a personal-owned repository (ORG-03 / D-ORG-04).
#[tokio::test]
async fn collab_crud_on_personal_repo() {
    assert!(
        false,
        "Wave 0: collaborator add/list/update/remove on personal repo (ORG-03 / D-ORG-04)"
    );
}

/// Collaborator CRUD on an org-owned repository (ORG-03 / D-ORG-04).
#[tokio::test]
async fn collab_crud_on_org_repo() {
    assert!(
        false,
        "Wave 0: collaborator add/list/update/remove on org-owned repo (ORG-03 / D-ORG-04)"
    );
}

/// Collaborator permission ladder: read | write | admin (ORG-03 / D-ORG-02c).
#[tokio::test]
async fn collab_permission_read_write_admin() {
    assert!(
        false,
        "Wave 0: collaborator permission must be read|write|admin (ORG-03 / D-ORG-02c)"
    );
}

/// Unauthorized private → soft `repo.not_found` on web RPC (ORG-04 / D-ORG-05 / T-10-01).
#[tokio::test]
async fn collab_unauthorized_private_soft_not_found_web() {
    assert!(
        false,
        "Wave 0: private non-grantee web path → repo.not_found (ORG-04 / D-ORG-05 / D-25)"
    );
}

/// Collaborator grant raises Member with member_base=none on private org repo (ORG-02/03 / T-10-02).
#[tokio::test]
async fn collab_raises_member_base_none_on_private_org_repo() {
    assert!(
        false,
        "Wave 0: Collaborator raise over Member base none on private org repo (ORG-02/03 / D-ORG-02b)"
    );
}

/// Visibility changes remain admin-gated with collaborators present (ORG-03).
#[tokio::test]
async fn collab_visibility_change_requires_admin() {
    assert!(
        false,
        "Wave 0: visibility mutate still requires admin when collaborators exist (ORG-03)"
    );
}
