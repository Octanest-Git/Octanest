//! Env-seeded initial admin (D-04 auto-verify).

use chrono::Utc;
use uuid::Uuid;

use crate::auth::hash_password_str;
use octanest_db::Database;

/// Create a single `is_admin` user when `OCTANEST_ADMIN_EMAIL` + `OCTANEST_ADMIN_PASSWORD`
/// are both set and `count_users() == 0`. Username `admin` if free else `admin1`.
/// Seeded admin is auto-verified (D-04).
pub async fn maybe_seed_admin(db: &Database) -> Result<(), String> {
    let email = match std::env::var("OCTANEST_ADMIN_EMAIL") {
        Ok(v) if !v.is_empty() => v.trim().to_ascii_lowercase(),
        _ => return Ok(()),
    };
    let password = match std::env::var("OCTANEST_ADMIN_PASSWORD") {
        Ok(v) if !v.is_empty() => v,
        _ => return Ok(()),
    };

    let count = db.count_users().await?;
    if count > 0 {
        return Ok(());
    }

    let username = match db.find_user_by_username("admin").await? {
        None => "admin".to_string(),
        Some(_) => "admin1".to_string(),
    };

    let password_hash = hash_password_str(&password).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    db.create_user(
        &id,
        &email,
        &username,
        Some(&password_hash),
        &username,
        "",
        None,
        true,
    )
    .await?;

    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&id, &now).await?;

    tracing::info!(
        username = %username,
        email = %email,
        "seeded initial admin user from OCTANEST_ADMIN_* (Phase 6 wizard owns interactive bootstrap)"
    );
    Ok(())
}
