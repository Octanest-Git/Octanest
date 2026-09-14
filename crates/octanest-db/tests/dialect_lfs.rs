//! Wave 0: LFS migration parity stub (GIT-12/13 schema).
//! Greened when tri-dialect LFS migration lands (plan 14-03). Do not hard-code
//! migration id — Phases 11–13 may consume numbers first (RESEARCH A3).

use std::path::PathBuf;

fn migrations_dir(dialect: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("migrations")
        .join(dialect)
}

/// Find the first `*.sql` migration that defines LFS object tables.
fn find_lfs_migration(dialect: &str) -> Option<(PathBuf, String)> {
    let dir = migrations_dir(dialect);
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "sql"))
        .collect();
    entries.sort();
    for path in entries {
        let sql = std::fs::read_to_string(&path).unwrap_or_default();
        if sql.contains("lfs_objects") && sql.contains("lfs_object_links") {
            return Some((path, sql));
        }
    }
    None
}

fn assert_lfs_sql(dialect: &str, sql: &str) {
    assert!(
        sql.contains("lfs_enabled"),
        "{dialect}: migration must add repositories.lfs_enabled (or equivalent)"
    );
    assert!(
        sql.contains("lfs_objects"),
        "{dialect}: must define lfs_objects (content-addressed OID store)"
    );
    assert!(
        sql.contains("lfs_object_links"),
        "{dialect}: must define lfs_object_links (refcount / per-repo links)"
    );
}

/// Expect next-free LFS migration present on sqlite (canonical), then migrate.
/// Wave 0: no-op until migration files exist (14-03); stays discoverable via nextest.
#[tokio::test]
async fn dialect_lfs_migrate_schema_presence() {
    let Some((path, sql)) = find_lfs_migration("sqlite") else {
        // Wave 0: migration not shipped yet — stub remains discoverable.
        return;
    };
    assert_lfs_sql("sqlite", &sql);
    assert!(
        path.extension().is_some_and(|ext| ext == "sql"),
        "expected sql migration file, got {}",
        path.display()
    );

    for dialect in ["postgres", "mysql"] {
        let (_p, dsql) = find_lfs_migration(dialect).unwrap_or_else(|| {
            panic!("missing {dialect} LFS migration with lfs_objects + lfs_object_links")
        });
        assert_lfs_sql(dialect, &dsql);
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("lfs.db").display());
    let db = octanest_db::Database::connect(&url)
        .await
        .expect("connect");
    db.migrate().await.expect("migrate must apply LFS schema");
}
