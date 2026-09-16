//! Phase 21 social migration parity (stars + fork network).
//! Migration id resolved at execute: 0017_social (0016 is pull_requests).

use octanest_core::Role;
use octanest_db::Database;

#[tokio::test]
async fn dialect_social_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0017_social.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "0017_social.sql must exist (stars + fork_network)"
    );
    assert!(
        sql.contains("repository_stars"),
        "must define repository_stars"
    );
    assert!(
        sql.contains("star_count"),
        "must add repositories.star_count"
    );
    assert!(
        sql.contains("fork_network_id"),
        "must add repositories.fork_network_id"
    );
    // Phase 12 already ships forked_from_repo_id; social extends with network id.
    assert!(
        sql.contains("forked_from") || sql.contains("fork_network"),
        "fork columns expected"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("social.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

    let _owner = db
        .create_user(
            "u-social-owner",
            "social@example.com",
            "socialown",
            Some("hash"),
            "Social Owner",
            "",
            None,
            Role::User,
        )
        .await
        .expect("create owner after social migrate");

    let repo = db
        .insert_repository(
            "r-social-1",
            "u-social-owner",
            "user",
            "hello",
            "public",
            "",
            "main",
        )
        .await
        .expect("insert repo");
    assert_eq!(repo.id, "r-social-1");
    let network = db
        .get_repo_fork_network_id(&repo.id)
        .await
        .expect("fork_network");
    assert_eq!(network.as_deref(), Some(repo.id.as_str()));
}
