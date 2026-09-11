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
            octanest_core::Role::User,
        )
        .await
        .expect("insert user");
    assert_eq!(user.email, "roundtrip@example.com");
    assert_eq!(user.password_hash.as_deref(), Some("$argon2id$test"));
    assert_eq!(user.role, octanest_core::Role::User);

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

#[tokio::test]
async fn migrate_email_token_and_verified_helpers() {
    let Some(url) = database_url() else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    let _guard = SERIAL.lock().await;

    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

    let user_id = "00000000-0000-4000-8000-000000000002";
    db.create_user(
        user_id,
        "verify-helpers@example.com",
        "verify-helpers",
        Some("$argon2id$test"),
        "Verify Helpers",
        "",
        None,
        octanest_core::Role::User,
    )
    .await
    .expect("insert user");

    let token_hash = "def0123456789abcdef0123456789abcdef0123456789abcdef0123456789ab";
    let otp_hash = "fed0123456789abcdef0123456789abcdef0123456789abcdef0123456789ab";
    let token = db
        .upsert_email_token(
            "00000000-0000-4000-8000-0000000000bb",
            user_id,
            "verify",
            token_hash,
            otp_hash,
            "2099-06-01T12:00:00Z",
            1,
        )
        .await
        .expect("upsert email token");
    assert_eq!(token.purpose, "verify");
    assert_eq!(token.user_id, user_id);
    assert_eq!(token.attempt_count, 0);
    assert_eq!(token.issue_count, 1);

    let by_token = db
        .find_email_token_by_token_hash(token_hash)
        .await
        .expect("find by token_hash")
        .expect("token row present");
    assert_eq!(by_token.id, token.id);
    assert_eq!(by_token.otp_hash, otp_hash);

    let by_otp = db
        .find_email_token_by_otp_hash(otp_hash)
        .await
        .expect("find by otp_hash")
        .expect("otp row present");
    assert_eq!(by_otp.token_hash, token_hash);

    let verified = db
        .set_email_verified_at(user_id, "2099-06-01T12:30:00Z")
        .await
        .expect("set email_verified_at");
    assert!(verified.email_verified_at.is_some());

    let cleared = db
        .clear_email_verified_at(user_id)
        .await
        .expect("clear email_verified_at");
    assert!(cleared.email_verified_at.is_none());
}

/// Wave 0 (06-00): `0006_bootstrap_flags` must add `allow_signup` + `must_change_credentials`.
/// RED until 06-01 lands the migration triple + DTO fields.
#[tokio::test]
async fn migrate_0006_bootstrap_flags_columns() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0006_bootstrap_flags.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "0006_bootstrap_flags.sql must exist (allow_signup + must_change_credentials)"
    );
    assert!(
        sql.contains("allow_signup"),
        "0006 must add instance_auth_settings.allow_signup"
    );
    assert!(
        sql.contains("must_change_credentials"),
        "0006 must add users.must_change_credentials"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("bootstrap_flags.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

    // Wave 0: fields land with 06-01 — intentional RED until AuthSettingsRow/UserRow expose them.
    let settings = db.get_auth_settings().await.expect("settings");
    let _ = &settings.provider_mode;
    let allow_signup: Option<bool> = None;
    assert_eq!(
        allow_signup,
        Some(false),
        "after migrate, allow_signup default false must be readable"
    );
}
