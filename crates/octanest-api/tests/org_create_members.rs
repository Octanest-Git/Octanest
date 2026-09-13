//! ORG-01 / ORG-02 Wave 0 stubs: org.create, members, member_base ACL effects.
//!
//! RED until org RPC + ACL land (10-02+). Do not implement production handlers here.
//! Threat: T-10-01 soft not_found for private deny; T-10-02 Member base none denies private.

/// Verified `org.create` reserves a slug shared with the user namespace (D-ORG-01).
#[tokio::test]
async fn org_create_reserves_shared_slug_namespace() {
    assert!(
        false,
        "Wave 0: org.create must reject slugs colliding with users.username (ORG-01 / D-ORG-01)"
    );
}

/// Creator of an org is Owner (ORG-01 / D-ORG-02a).
#[tokio::test]
async fn org_create_creator_is_owner() {
    assert!(
        false,
        "Wave 0: org.create must add the creator as Owner (ORG-01 / D-ORG-02a)"
    );
}

/// `members.add` by username adds an existing instance user (ORG-01 / D-ORG-03).
#[tokio::test]
async fn org_members_add_by_username() {
    assert!(
        false,
        "Wave 0: members.add by username must link an existing user (ORG-01 / D-ORG-03)"
    );
}

/// `members.updateRole` changes Owner/Admin/Member (ORG-02 / D-ORG-02a).
#[tokio::test]
async fn org_members_update_role() {
    assert!(
        false,
        "Wave 0: members.updateRole must set Owner|Admin|Member (ORG-02 / D-ORG-02a)"
    );
}

/// Demoting or removing the last Owner → `org.last_owner` (ORG-01).
#[tokio::test]
async fn org_members_last_owner_demote_or_remove_rejected() {
    assert!(
        false,
        "Wave 0: last Owner demote/remove → org.last_owner (ORG-01)"
    );
}

/// Member with member_base=none cannot read private org repo (ORG-02 / D-ORG-02b / T-10-02).
#[tokio::test]
async fn org_member_base_none_denies_private_repo_read() {
    assert!(
        false,
        "Wave 0: Member + member_base none → deny private org repo read (ORG-02 / D-ORG-02b)"
    );
}

/// Member with member_base=read can read private org repo (ORG-02 / D-ORG-02b).
#[tokio::test]
async fn org_member_base_read_allows_private_repo_read() {
    assert!(
        false,
        "Wave 0: Member + member_base read → can read private org repo (ORG-02 / D-ORG-02b)"
    );
}

/// Member with member_base=write can write private org repo (ORG-02 / D-ORG-02b).
#[tokio::test]
async fn org_member_base_write_allows_private_repo_write() {
    assert!(
        false,
        "Wave 0: Member + member_base write → can write private org repo (ORG-02 / D-ORG-02b)"
    );
}
