//! Wave 0 / 08-03: `0008_pats` + personal_access_tokens / personal_access_token_repos.
//!
//! RED until tri-dialect migration lands. Mirror dialect_repositories pattern.

/// Expect sqlite `0008_pats.sql` with PAT tables (hash-at-rest; no plaintext column).
#[tokio::test]
async fn dialect_pats_migrate_0008_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0008_pats.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "Wave 0: 0008_pats.sql must exist (personal_access_tokens + personal_access_token_repos)"
    );
    assert!(
        sql.contains("personal_access_tokens"),
        "Wave 0: 0008 must define personal_access_tokens"
    );
    assert!(
        sql.contains("personal_access_token_repos"),
        "Wave 0: 0008 must define personal_access_token_repos (FG selected repos)"
    );
    assert!(
        sql.contains("token_hash") || sql.contains("hash"),
        "Wave 0: 0008 must store token_hash only (no plaintext at rest)"
    );

    assert!(
        false,
        "Wave 0: Database PAT APIs + migrate 0008_pats not wired yet"
    );
}

/// Tri-dialect parity: postgres and mysql siblings must exist alongside sqlite.
#[test]
fn dialect_pats_tri_dialect_files() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
    for dialect in ["sqlite", "postgres", "mysql"] {
        let path = format!("{root}/{dialect}/0008_pats.sql");
        let sql = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !sql.is_empty(),
            "Wave 0: missing {path} — tri-dialect 0008_pats required"
        );
        assert!(
            sql.contains("personal_access_tokens"),
            "Wave 0: {dialect} 0008_pats must mention personal_access_tokens"
        );
        assert!(
            sql.contains("personal_access_token_repos"),
            "Wave 0: {dialect} 0008_pats must mention personal_access_token_repos"
        );
    }
}
