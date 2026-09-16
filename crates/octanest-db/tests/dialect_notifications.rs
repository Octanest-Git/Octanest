//! Phase 17 Wave 0 — notifications migration parity stub (sqlite/postgres/mysql).
//!
//! Greened by 17-01 when `00NN_notifications.sql` lands for all dialects.

#[tokio::test]
#[ignore = "Wave 0 stub — greened when notifications migration exists for all dialects"]
async fn dialect_notifications_migration_module_present() {
    let sqlite = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0017_notifications.sql"
    );
    let postgres = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/postgres/0017_notifications.sql"
    );
    let mysql = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/mysql/0017_notifications.sql"
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
}
