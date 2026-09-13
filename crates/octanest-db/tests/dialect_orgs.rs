//! 10-02: `0010_orgs_acl` + organizations / members / invites / collaborators.

use octanest_core::Role;
use octanest_db::{resolve_dialect_from_env, Database, DbPool};
use sqlx::Row;

/// Expect sqlite `0010_orgs_acl.sql` with org ACL tables + repositories.owner_type,
/// then insert organization + Owner membership round-trip.
#[tokio::test]
async fn dialect_orgs_migrate_0010_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0010_orgs_acl.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "0010_orgs_acl.sql must exist (organizations + members + invites + collaborators)"
    );
    assert!(
        sql.contains("organizations"),
        "0010 must define organizations"
    );
    assert!(
        sql.contains("organization_members"),
        "0010 must define organization_members"
    );
    assert!(
        sql.contains("organization_invites"),
        "0010 must define organization_invites"
    );
    assert!(
        sql.contains("repository_collaborators"),
        "0010 must define repository_collaborators"
    );
    assert!(
        sql.contains("owner_type"),
        "0010 must add repositories.owner_type"
    );
    assert!(
        sql.contains("token_hash") || sql.contains("member_base_permission"),
        "0010 must include invite token_hash and/or member_base_permission"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("orgs.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate 0010_orgs_acl");

    let owner = db
        .create_user(
            "u-org-owner",
            "orgowner@example.com",
            "orgowner",
            Some("hash"),
            "Org Owner",
            "",
            None,
            Role::User,
        )
        .await
        .expect("create owner user");

    let org = db
        .insert_organization("o-acme", "acme", "Acme Corp", "none")
        .await
        .expect("insert organization");
    assert_eq!(org.id, "o-acme");
    assert_eq!(org.slug, "acme");
    assert_eq!(org.display_name, "Acme Corp");
    assert_eq!(org.member_base_permission, "none");

    let by_slug = db
        .find_organization_by_slug("ACME")
        .await
        .expect("find by slug")
        .expect("org present");
    assert_eq!(by_slug.id, "o-acme");

    let member = db
        .insert_org_owner_membership(&org.id, &owner.id)
        .await
        .expect("insert Owner membership");
    assert_eq!(member.org_id, org.id);
    assert_eq!(member.user_id, owner.id);
    assert_eq!(member.role, "owner");

    let found = db
        .find_org_member(&org.id, &owner.id)
        .await
        .expect("find member")
        .expect("membership present");
    assert_eq!(found.role, "owner");

    // Polymorphic owner_type column exists and defaults to user for legacy inserts.
    let repo = db
        .insert_repository("r-org-1", &owner.id, "demo", "private", "", "main")
        .await
        .expect("insert user-owned repo after 0010");
    assert_eq!(repo.owner_id, owner.id);

    let dialect = resolve_dialect_from_env(&url).expect("dialect");
    let pool = DbPool::connect(&url, dialect).await.expect("pool");
    let DbPool::Sqlite(p) = pool else {
        panic!("expected sqlite pool");
    };
    let row = sqlx::query("SELECT owner_type FROM repositories WHERE id = ?1")
        .bind(&repo.id)
        .fetch_one(&p)
        .await
        .expect("select owner_type");
    let owner_type: String = row.try_get("owner_type").expect("owner_type col");
    assert_eq!(owner_type, "user");
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
            "missing {path} — tri-dialect 0010_orgs_acl required"
        );
        assert!(
            sql.contains("organizations"),
            "{dialect} 0010_orgs_acl must mention organizations"
        );
        assert!(
            sql.contains("organization_members"),
            "{dialect} 0010_orgs_acl must mention organization_members"
        );
        assert!(
            sql.contains("organization_invites"),
            "{dialect} 0010_orgs_acl must mention organization_invites"
        );
        assert!(
            sql.contains("repository_collaborators"),
            "{dialect} 0010_orgs_acl must mention repository_collaborators"
        );
        assert!(
            sql.contains("owner_type"),
            "{dialect} 0010_orgs_acl must mention owner_type"
        );
        assert!(
            sql.contains("member_base_permission"),
            "{dialect} 0010_orgs_acl must mention member_base_permission"
        );
        assert!(
            sql.contains("token_hash"),
            "{dialect} 0010_orgs_acl must mention invite token_hash"
        );
    }
}
