//! 20-02: `0015_packages` + packages / versions / blobs / refs / quota tables.

use octanest_core::Role;
use octanest_db::Database;

/// Expect sqlite `0015_packages.sql` with packages domain tables, then migrate applies.
#[tokio::test]
async fn dialect_packages_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0015_packages.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "0015_packages.sql must exist (packages domain)"
    );
    for table in [
        "packages",
        "package_versions",
        "package_blobs",
        "package_blob_refs",
        "package_quota_overrides",
    ] {
        assert!(
            sql.contains(table),
            "0012 must define {}",
            table
        );
    }
    assert!(
        sql.contains("owner_type") && sql.contains("owner_id") && sql.contains("format"),
        "packages must carry owner + format (D-PKG-02)"
    );
    assert!(
        sql.contains("repository_id"),
        "packages must allow optional repository link"
    );
    assert!(
        sql.contains("refcount"),
        "package_blobs must track refcount (D-PKG-08)"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("packages.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

    // Prove DB is usable after packages migration (insert a user — init schema intact).
    let _owner = db
        .create_user(
            "u-pkg-owner",
            "pkgowner@example.com",
            "pkgowner",
            Some("hash"),
            "Pkg Owner",
            "",
            None,
            Role::User,
        )
        .await
        .expect("create owner after packages migrate");
}
