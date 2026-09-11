//! Shared helpers for octanest-api integration tests.

#![allow(dead_code)]

use std::sync::{Mutex, MutexGuard};

use octanest_api::auth::hash_password_str;
use octanest_db::Database;
use uuid::Uuid;

/// Serialize tests that mutate `OCTANEST_ADMIN_*` / `OCTANEST_ALLOW_SIGNUP` (process-wide env).
pub fn lock_admin_env() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Create a verified `sys-admin` when the users table is empty so open signup is allowed.
/// Avoids toggling `OCTANEST_ADMIN_*` (racy under parallel tests).
pub async fn unlock_signup(db: &Database) {
    let count = db.count_users().await.expect("count_users");
    if count > 0 {
        return;
    }
    let id = Uuid::new_v4().to_string();
    let email = format!("sysadmin-{id}@example.com");
    let username = format!("sys-{}", &id[..8]);
    let password_hash = hash_password_str("test-sysadmin-pass").expect("hash");
    db.create_user(
        &id,
        &email,
        &username,
        Some(&password_hash),
        &username,
        "",
        None,
        octanest_core::Role::SysAdmin,
    )
    .await
    .expect("create sys-admin");
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&id, &now)
        .await
        .expect("verify sys-admin");
}
