//! 12-02: `0016_pull_requests` schema presence + migrate (PR domain).

use octanest_core::Role;
use octanest_db::Database;

#[tokio::test]
async fn dialect_pulls_migrate_0016_schema_presence() {
    for (label, path) in [
        (
            "sqlite",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/migrations/sqlite/0016_pull_requests.sql"
            ),
        ),
        (
            "postgres",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/migrations/postgres/0016_pull_requests.sql"
            ),
        ),
        (
            "mysql",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/migrations/mysql/0016_pull_requests.sql"
            ),
        ),
    ] {
        let sql = std::fs::read_to_string(path).unwrap_or_default();
        assert!(!sql.is_empty(), "{label} 0016_pull_requests.sql must exist");
        for needle in [
            "pull_requests",
            "pull_comments",
            "pull_reviews",
            "pull_review_requests",
            "pull_labels",
            "pull_assignees",
            "allow_merge_commit",
            "allow_squash_merge",
            "allow_rebase_merge",
            "forked_from_repo_id",
        ] {
            assert!(
                sql.contains(needle),
                "{label} 0016 must define {needle}"
            );
        }
        assert!(
            sql.contains("'pr'") || sql.contains("\"pr\"") || sql.contains(", 'pr'") || sql.contains("pr'"),
            "{label} 0016 must expand issue_links kind to include pr"
        );
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pulls.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate 0016_pull_requests");

    let author = db
        .create_user(
            "u-pull-author",
            "pullauthor@example.com",
            "pullauthor",
            Some("hash"),
            "Pull Author",
            "",
            None,
            Role::User,
        )
        .await
        .expect("create author");

    let repo = db
        .insert_repository(
            "r-pulls-1",
            &author.id,
            "user",
            "pulls-demo",
            "private",
            "",
            "main",
        )
        .await
        .expect("insert repo");

    let settings = db
        .get_repo_merge_settings(&repo.id)
        .await
        .expect("merge settings defaults");
    assert!(settings.allow_merge_commit);
    assert!(settings.allow_squash_merge);
    assert!(settings.allow_rebase_merge);

    let n = db
        .allocate_next_issue_number(&repo.id)
        .await
        .expect("allocate shared #N");
    let pull = db
        .insert_pull(
            "p-1",
            &repo.id,
            n,
            "First PR",
            "body",
            &author.id,
            "main",
            "abc",
            &repo.id,
            "feature",
            "def",
            false,
        )
        .await
        .expect("insert pull");
    assert_eq!(pull.number, n);
    assert_eq!(pull.state, "open");

    let found = db
        .find_pull_by_repo_number(&repo.id, n)
        .await
        .expect("find")
        .expect("present");
    assert_eq!(found.id, "p-1");

    let (list, total) = db
        .list_pulls_for_repo(&repo.id, Some("open"), 0, 20)
        .await
        .expect("list");
    assert_eq!(total, 1);
    assert_eq!(list.len(), 1);

    // Postgres maps TIMESTAMPTZ → String via PULL_COLS_PG; keep the cast list in sync.
    let pulls_src = include_str!("../src/pulls.rs");
    assert!(
        pulls_src.contains("PULL_COLS_PG"),
        "Postgres pull SELECTs must use PULL_COLS_PG (TIMESTAMPTZ → text)"
    );
    assert!(
        pulls_src.contains("to_char(created_at AT TIME ZONE 'UTC'")
            && pulls_src.contains("AS created_at"),
        "PULL_COLS_PG must cast created_at for sqlx String decode"
    );
}
