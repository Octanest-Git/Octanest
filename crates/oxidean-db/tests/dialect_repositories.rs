//! 07-02: `0007_repositories` + default_branch / default_visibility + insert/get API.

use oxidean_core::Role;
use oxidean_db::Database;

/// Expect sqlite `0007_repositories.sql` with repositories table + settings columns,
/// then insert_repository + get by owner+name for non-deleted rows.
#[tokio::test]
async fn migrate_0007_repositories_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0007_repositories.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "0007_repositories.sql must exist (repositories + soft-delete)"
    );
    assert!(
        sql.contains("repositories") || sql.contains("CREATE TABLE"),
        "0007 must define repositories table"
    );
    assert!(
        sql.contains("default_branch"),
        "0007 must include default_branch (repo and/or user column)"
    );
    assert!(
        sql.contains("default_visibility"),
        "0007 must include instance default_visibility"
    );
    assert!(
        sql.contains("deleted_at") || sql.contains("soft"),
        "0007 must support soft-delete among non-deleted uniqueness"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("repositories.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

    let settings = db.get_auth_settings().await.expect("settings");
    assert_eq!(
        settings.default_visibility, "public",
        "instance default_visibility defaults to public (D-08)"
    );

    let owner = db
        .create_user(
            "u-repo-owner",
            "owner@example.com",
            "repoowner",
            Some("hash"),
            "Repo Owner",
            "",
            None,
            Role::User,
        )
        .await
        .expect("create owner");
    assert_eq!(
        owner.default_branch, "main",
        "user default_branch defaults to main (D-09)"
    );

    let inserted = db
        .insert_repository(
            "r-demo",
            &owner.id,
            "user",
            "my_app",
            "public",
            "demo repo",
            &owner.default_branch,
        )
        .await
        .expect("insert_repository");
    assert_eq!(inserted.name, "my_app");
    assert!(inserted.deleted_at.is_none());

    let found = db
        .find_repository_by_owner_name(&owner.id, "my_app")
        .await
        .expect("find")
        .expect("row present");
    assert_eq!(found.id, "r-demo");
    assert_eq!(found.visibility, "public");
    assert_eq!(found.description, "demo repo");

    // Case-insensitive name lookup among non-deleted
    let found_ci = db
        .find_repository_by_owner_name(&owner.id, "My_App")
        .await
        .expect("find ci")
        .expect("row present case-insensitive");
    assert_eq!(found_ci.id, "r-demo");
}

/// Tri-dialect parity: postgres and mysql siblings must exist alongside sqlite.
#[test]
fn migrate_0007_repositories_tri_dialect_files() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
    for dialect in ["sqlite", "postgres", "mysql"] {
        let path = format!("{root}/{dialect}/0007_repositories.sql");
        let sql = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !sql.is_empty(),
            "missing {path} — tri-dialect 0007_repositories required"
        );
        assert!(
            sql.contains("default_branch"),
            "{dialect} 0007 must mention default_branch"
        );
        assert!(
            sql.contains("default_visibility"),
            "{dialect} 0007 must mention default_visibility"
        );
        assert!(
            sql.contains("deleted_at"),
            "{dialect} 0007 must mention deleted_at soft-delete"
        );
    }
}
