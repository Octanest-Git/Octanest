//! Wave 0 / Phase 15: tri-dialect releases + redirects migration parity stub.
//! Turned green in 15-01 when `0012_releases_redirects` (next-free) lands.

use octanest_db::Database;

/// Expect next-free `00NN_releases_redirects.sql` with releases, release_assets,
/// repository_redirects across sqlite (migration file presence + migrate round-trip).
#[tokio::test]
#[ignore = "Wave 0: green in 15-01"]
async fn dialect_releases_migrate_schema_presence() {
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations/sqlite");
    let entries = std::fs::read_dir(migration_dir).expect("sqlite migrations dir");
    let mut found = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains("releases_redirects") || name.contains("releases") {
            found = Some(name);
            break;
        }
    }
    let name = found.expect(
        "Wave 0 stub: next-free 00NN_releases_redirects.sql must exist (releases + release_assets + repository_redirects)",
    );
    let path = format!("{migration_dir}/{name}");
    let sql = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        sql.contains("releases"),
        "{name} must define releases table"
    );
    assert!(
        sql.contains("release_assets"),
        "{name} must define release_assets"
    );
    assert!(
        sql.contains("repository_redirects"),
        "{name} must define repository_redirects"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("releases.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate()
        .await
        .expect("migrate releases_redirects tri-dialect schema");

    // Schema presence probe — helpers land in 15-01.
    assert!(
        false,
        "Wave 0 stub: after migrate, releases/release_assets/repository_redirects must be insertable (15-01)"
    );
}
