//! Wave 0 / 07-02: `0007_repositories` + default_branch / default_visibility columns.
//!
//! RED until tri-dialect migration lands. Asserts file presence + column names per RESEARCH.

use octanest_db::Database;

/// Expect sqlite `0007_repositories.sql` with repositories table + settings columns.
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

    // After migrate, columns must be readable via Database API (lands with repositories.rs).
    assert!(
        false,
        "Wave 0: Database repositories + user default_branch / instance default_visibility APIs not wired"
    );
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
            "Wave 0: missing {path} — tri-dialect 0007_repositories required"
        );
        assert!(
            sql.contains("default_branch"),
            "Wave 0: {dialect} 0007 must mention default_branch"
        );
        assert!(
            sql.contains("default_visibility"),
            "Wave 0: {dialect} 0007 must mention default_visibility"
        );
    }
}
