//! DATABASE_URL-gated auth migrate + user/session round-trip.
//! CI sets DATABASE_URL per dialect leg; local runs skip when unset.

use octanest_db::Database;

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn database_url() -> Option<String> {
    match std::env::var("DATABASE_URL") {
        Ok(url) if !url.is_empty() => Some(url),
        _ => None,
    }
}

#[tokio::test]
async fn migrate_auth_and_user_round_trip() {
    let Some(url) = database_url() else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    let _guard = SERIAL.lock().await;

    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

    let settings = db.get_auth_settings().await.expect("auth settings seed");
    assert_eq!(settings.provider_mode, "local");
    assert_eq!(settings.email_provider, "log");

    let user_id = "00000000-0000-4000-8000-000000000001";
    let user = db
        .create_user(
            user_id,
            "roundtrip@example.com",
            "roundtrip-user",
            Some("$argon2id$test"),
            "Round Trip",
            "",
            None,
            false,
        )
        .await
        .expect("insert user");
    assert_eq!(user.email, "roundtrip@example.com");
    assert_eq!(user.password_hash.as_deref(), Some("$argon2id$test"));

    let token_hash = "abc0123456789abcdef0123456789abcdef0123456789abcdef0123456789ab";
    db.create_session(
        "00000000-0000-4000-8000-0000000000aa",
        user_id,
        token_hash,
        "2099-01-01T00:00:00Z",
        false,
    )
    .await
    .expect("create session");

    let session = db
        .find_session_by_token_hash(token_hash)
        .await
        .expect("find session")
        .expect("session present");
    assert_eq!(session.user_id, user_id);

    let deleted = db
        .delete_sessions_for_user(user_id)
        .await
        .expect("delete sessions for user");
    assert!(deleted >= 1);

    let gone = db
        .find_session_by_token_hash(token_hash)
        .await
        .expect("find after delete");
    assert!(gone.is_none());
}
