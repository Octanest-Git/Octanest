//! Wave 0 / 10-02: `0010_orgs_acl` + organizations / members / invites / collaborators.
//!
//! RED until tri-dialect migration lands. Mirror dialect_pats pattern.
//! Prefer 0010 because Phase 09 owns 0009_ssh_keys (D-ORG migration note).

/// Expect sqlite `0010_orgs_acl.sql` with org ACL tables + repositories.owner_type.
#[tokio::test]
async fn dialect_orgs_migrate_0010_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0010_orgs_acl.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "Wave 0: 0010_orgs_acl.sql must exist (organizations + members + invites + collaborators)"
    );
    assert!(
        sql.contains("organizations"),
        "Wave 0: 0010 must define organizations"
    );
    assert!(
        sql.contains("organization_members"),
        "Wave 0: 0010 must define organization_members"
    );
    assert!(
        sql.contains("organization_invites"),
        "Wave 0: 0010 must define organization_invites"
    );
    assert!(
        sql.contains("repository_collaborators"),
        "Wave 0: 0010 must define repository_collaborators"
    );
    assert!(
        sql.contains("owner_type"),
        "Wave 0: 0010 must add repositories.owner_type"
    );
    assert!(
        sql.contains("token_hash") || sql.contains("member_base_permission"),
        "Wave 0: 0010 must include invite token_hash and/or member_base_permission"
    );

    assert!(
        false,
        "Wave 0: Database org APIs + migrate 0010_orgs_acl not wired yet"
    );
}

/// Tri-dialect parity: postgres and mysql siblings must exist alongside sqlite.
#[test]
fn dialect_orgs_tri_dialect_files() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
    for dialect in ["sqlite", "postgres", "mysql"] {
        let path = format!("{root}/{dialect}/0010_orgs_acl.sql");
        let sql = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !sql.is_empty(),
            "Wave 0: missing {path} — tri-dialect 0010_orgs_acl required"
        );
        assert!(
            sql.contains("organizations"),
            "Wave 0: {dialect} 0010_orgs_acl must mention organizations"
        );
        assert!(
            sql.contains("organization_members"),
            "Wave 0: {dialect} 0010_orgs_acl must mention organization_members"
        );
        assert!(
            sql.contains("organization_invites"),
            "Wave 0: {dialect} 0010_orgs_acl must mention organization_invites"
        );
        assert!(
            sql.contains("repository_collaborators"),
            "Wave 0: {dialect} 0010_orgs_acl must mention repository_collaborators"
        );
        assert!(
            sql.contains("owner_type"),
            "Wave 0: {dialect} 0010_orgs_acl must mention owner_type"
        );
    }
}
