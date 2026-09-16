//! Wave 0: expect `0016_pull_requests` tri-dialect (greened in 12-02).

#[tokio::test]
#[ignore = "Wave 0 stub — greened when 0016_pull_requests lands"]
async fn dialect_pulls_migrate_0016_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0016_pull_requests.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "0016_pull_requests.sql must exist"
    );
    for needle in [
        "pull_requests",
        "pull_comments",
        "pull_reviews",
        "pull_review_requests",
        "allow_merge_commit",
        "allow_squash_merge",
        "allow_rebase_merge",
        "forked_from_repo_id",
    ] {
        assert!(sql.contains(needle), "0016 must define {needle}");
    }
}
