//! 17-01: `0018_notifications` migration parity across dialects.

use oxidean_db::Database;

#[tokio::test]
async fn dialect_notifications_migration_module_present() {
    let sqlite = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0018_notifications.sql"
    );
    let postgres = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/postgres/0018_notifications.sql"
    );
    let mysql = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/mysql/0018_notifications.sql"
    );
    for path in [sqlite, postgres, mysql] {
        let sql = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            !sql.is_empty(),
            "notifications migration must exist at {path}"
        );
        assert!(
            sql.contains("notifications"),
            "{path} must define notifications table"
        );
        assert!(
            sql.contains("recipient_id"),
            "{path} must include recipient_id"
        );
        assert!(
            sql.contains("read_at"),
            "{path} must include read_at (null = unread)"
        );
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("notif.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate 0018_notifications");
}
