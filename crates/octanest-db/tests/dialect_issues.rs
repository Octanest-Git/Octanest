//! 11-02: `0011_issues` + issues / counters / comments / revisions / labels / assignees / reactions / links.

use octanest_core::Role;
use octanest_db::Database;

/// Expect sqlite `0011_issues.sql` with issue domain tables, migrate, and prove
/// monotonic `#N` allocation that does not reclaim after hard-delete (D-ISS-01).
#[tokio::test]
async fn dialect_issues_migrate_0011_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0011_issues.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "0011_issues.sql must exist (issues + counters + comments + revisions + labels + assignees + reactions + links)"
    );
    for needle in [
        "issues",
        "issue_counters",
        "issue_comments",
        "issue_revisions",
        "comment_revisions",
        "labels",
        "repo_hidden_labels",
        "issue_labels",
        "issue_assignees",
        "issue_reactions",
        "comment_reactions",
        "issue_links",
    ] {
        assert!(
            sql.contains(needle),
            "0011 must define {needle}"
        );
    }
    assert!(
        sql.contains("max_number"),
        "0011 issue_counters must track max_number"
    );
    assert!(
        sql.contains("pr_stub") || sql.contains("'pr_stub'"),
        "0011 issue_links must allow pr_stub kind"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("issues.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate 0011_issues");

    let author = db
        .create_user(
            "u-issue-author",
            "issueauthor@example.com",
            "issueauthor",
            Some("hash"),
            "Issue Author",
            "",
            None,
            Role::User,
        )
        .await
        .expect("create author");

    let repo = db
        .insert_repository(
            "r-issues-1",
            &author.id,
            "user",
            "issues-demo",
            "private",
            "",
            "main",
        )
        .await
        .expect("insert repo");

    let first = db
        .insert_issue(
            "i-1",
            &repo.id,
            &author.id,
            "First issue",
            "body one",
        )
        .await
        .expect("insert first issue");
    assert_eq!(first.number, 1);
    assert_eq!(first.state, "open");
    assert_eq!(first.title, "First issue");

    let second = db
        .insert_issue(
            "i-2",
            &repo.id,
            &author.id,
            "Second issue",
            "body two",
        )
        .await
        .expect("insert second issue");
    assert_eq!(second.number, 2);

    db.delete_issue(&first.id)
        .await
        .expect("hard-delete first issue");

    let third = db
        .insert_issue(
            "i-3",
            &repo.id,
            &author.id,
            "Third issue",
            "body three",
        )
        .await
        .expect("insert third after delete");
    assert_eq!(
        third.number, 3,
        "D-ISS-01: hard-delete must not reclaim #N (expected 3, got {})",
        third.number
    );

    let label = db
        .insert_label(
            "l-bug",
            "bug",
            "d73a4a",
            "A bug",
            None,
            Some(&repo.id),
        )
        .await
        .expect("insert repo-local label");
    assert_eq!(label.name, "bug");
    assert_eq!(label.repo_id.as_deref(), Some(repo.id.as_str()));

    db.set_issue_labels(&third.id, &["l-bug".to_string()])
        .await
        .expect("assign label");
    db.set_issue_assignees(&third.id, &[author.id.clone()])
        .await
        .expect("assign assignee");
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
            "missing {path} — tri-dialect 0011_issues required"
        );
        for needle in [
            "issues",
            "issue_counters",
            "issue_comments",
            "issue_revisions",
            "comment_revisions",
            "labels",
            "repo_hidden_labels",
            "issue_labels",
            "issue_assignees",
            "issue_reactions",
            "comment_reactions",
            "issue_links",
            "max_number",
        ] {
            assert!(
                sql.contains(needle),
                "{dialect} 0011_issues must mention {needle}"
            );
        }
        assert!(
            sql.contains("pr_stub") || sql.contains("'pr_stub'"),
            "{dialect} 0011_issues must mention pr_stub"
        );
    }
}
