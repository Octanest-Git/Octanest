//! Branch protection + commit_statuses migration parity (ORG-05/06, D-01, D-11).
//! Wave 0 stub — greened when 0017 (or next-free) migrations land in 13-02/13-04.

use std::path::PathBuf;

fn migrations_dir(dialect: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("migrations")
        .join(dialect)
}

fn find_protection_migration(dialect: &str) -> Option<(PathBuf, String)> {
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
        if sql.contains("branch_protection_rules") {
            return Some((path, sql));
        }
    }
    None
}

fn assert_protection_sql(dialect: &str, sql: &str) {
    assert!(
        sql.contains("branch_protection_rules"),
        "{dialect}: must define branch_protection_rules"
    );
    assert!(
        sql.contains("pattern") || sql.contains("branch_pattern"),
        "{dialect}: rule must include branch pattern column"
    );
}

#[tokio::test]
#[ignore = "Wave 0 RED — migrations land in 13-02/13-04"]
async fn dialect_branch_protection_migrate_schema_presence() {
    let (path, sql) =
        find_protection_migration("sqlite").expect("sqlite branch_protection migration");
    assert_protection_sql("sqlite", &sql);
    assert!(path.extension().is_some_and(|ext| ext == "sql"));

    for dialect in ["postgres", "mysql"] {
        let (_p, dsql) = find_protection_migration(dialect)
            .unwrap_or_else(|| panic!("missing {dialect} branch_protection migration"));
        assert_protection_sql(dialect, &dsql);
        assert!(
            dsql.contains("commit_statuses")
                || find_protection_migration(dialect)
                    .map(|(_, s)| s.contains("commit_statuses"))
                    .unwrap_or(false),
            "{dialect}: commit_statuses expected in protection or follow-on migration"
        );
    }

    assert!(
        false,
        "TODO 13-02: green dialect_branch_protection after tri-dialect migrations exist"
    );
}
