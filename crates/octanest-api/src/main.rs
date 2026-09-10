use octanest_api::auth::hash_password_str;
use octanest_api::build_cors;
use octanest_db::Database;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

/// Create a single `is_admin` user when `OCTANEST_ADMIN_EMAIL` + `OCTANEST_ADMIN_PASSWORD`
/// are both set and `count_users() == 0`. Username `admin` if free else `admin1`.
async fn maybe_seed_admin(db: &Database) -> Result<(), String> {
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
    tracing::info!(
        username = %username,
        email = %email,
        "seeded initial admin user from OCTANEST_ADMIN_* (Phase 6 wizard owns interactive bootstrap)"
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let env_name = std::env::var("OCTANEST_ENV").unwrap_or_else(|_| "development".into());
    let cors_origins = std::env::var("OCTANEST_CORS_ORIGINS").ok();
    let cors = match build_cors(&env_name, cors_origins.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cors config error: {e}");
            std::process::exit(1);
        }
    };

    let database_url = std::env::var("DATABASE_URL").ok().filter(|v| !v.is_empty());
    let db = match &database_url {
        Some(url) => {
            let dialect = match octanest_db::resolve_dialect_from_env(url) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("database config error: {e}");
                    std::process::exit(1);
                }
            };
            let db = match Database::from_env().await {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("database connect failed: {e}");
                    std::process::exit(1);
                }
            };
            tracing::info!("database dialect: {}", dialect.as_str());
            db
        }
        None => {
            tracing::warn!("DATABASE_URL not set; running without a database pool");
            Database::skipped()
        }
    };

    if database_url.is_some() {
        let auto_migrate = std::env::var("OCTANEST_AUTO_MIGRATE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        if auto_migrate {
            if let Err(e) = db.migrate().await {
                eprintln!("migration failed: {e}");
                std::process::exit(1);
            }
        } else {
            tracing::info!(
                "OCTANEST_AUTO_MIGRATE=false; run `make db-migrate` to apply migrations"
            );
        }

        // Optional first-admin seed (T-04-13). Phase 6 owns the interactive wizard —
        // this path only runs when both env vars are set and the users table is empty.
        if let Err(e) = maybe_seed_admin(&db).await {
            eprintln!("admin seed failed: {e}");
            std::process::exit(1);
        }
    }

    let bind = std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .unwrap_or_else(|e| {
            eprintln!("bind {bind}: {e}");
            std::process::exit(1);
        });
    tracing::info!("octanest-api listening on {bind}");

    let email = if database_url.is_some() {
        match db.get_auth_settings().await {
            Ok(settings) => octanest_api::email::build_email_sender_for_settings(&settings),
            Err(e) => {
                tracing::warn!(error = %e, "auth settings unavailable at boot; using ENV email sender");
                octanest_api::email::build_email_sender_from_env()
            }
        }
    } else {
        octanest_api::email::build_email_sender_from_env()
    };
    let state = octanest_api::AppState::new(db, email, env_name);
    let app = octanest_api::router_with_state(state, cors);
    axum::serve(listener, app).await.expect("server error");
}
