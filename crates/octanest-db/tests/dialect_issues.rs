//! Wave 0 / 11-02: `0011_issues` + issues / counters / comments / revisions / labels / assignees / reactions / links.
//!
//! RED until tri-dialect migration lands. Mirror dialect_orgs pattern.
//! Migration id is 0011_issues after 0010_orgs_acl.

/// Expect sqlite `0011_issues.sql` with issue domain tables.
#[tokio::test]
async fn dialect_issues_migrate_0011_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0011_issues.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "Wave 0: 0011_issues.sql must exist (issues + counters + comments + revisions + labels + assignees + reactions + links)"
    );
    assert!(
        sql.contains("issues"),
        "Wave 0: 0011 must define issues"
    );
    assert!(
        sql.contains("issue_counters"),
        "Wave 0: 0011 must define issue_counters"
    );
    assert!(
        sql.contains("comment"),
        "Wave 0: 0011 must define comments (or issue_comments)"
    );
    assert!(
        sql.contains("revision") || sql.contains("history"),
        "Wave 0: 0011 must define revisions/history tables"
    );
    assert!(
        sql.contains("label"),
        "Wave 0: 0011 must define labels"
    );
    assert!(
        sql.contains("assignee"),
        "Wave 0: 0011 must define assignees"
    );
    assert!(
        sql.contains("reaction"),
        "Wave 0: 0011 must define reactions"
    );
    assert!(
        sql.contains("link"),
        "Wave 0: 0011 must define issue/PR link stubs"
    );

    assert!(
        false,
        "Wave 0: Database issue APIs + migrate 0011_issues not wired yet"
    );
}

/// Tri-dialect parity: postgres and mysql siblings must exist alongside sqlite.
#[test]
fn dialect_issues_tri_dialect_files() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
    for dialect in ["sqlite", "postgres", "mysql"] {
        let path = format!("{root}/{dialect}/0011_issues.sql");
        let sql = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !sql.is_empty(),
            "Wave 0: missing {path} — tri-dialect 0011_issues required"
        );
        assert!(
            sql.contains("issues"),
            "Wave 0: {dialect} 0011_issues must mention issues"
        );
        assert!(
            sql.contains("issue_counters"),
            "Wave 0: {dialect} 0011_issues must mention issue_counters"
        );
        assert!(
            sql.contains("comment"),
            "Wave 0: {dialect} 0011_issues must mention comments"
        );
        assert!(
            sql.contains("label"),
            "Wave 0: {dialect} 0011_issues must mention labels"
        );
        assert!(
            sql.contains("assignee"),
            "Wave 0: {dialect} 0011_issues must mention assignees"
        );
        assert!(
            sql.contains("reaction"),
            "Wave 0: {dialect} 0011_issues must mention reactions"
        );
        assert!(
            sql.contains("link"),
            "Wave 0: {dialect} 0011_issues must mention links"
        );
    }
}
