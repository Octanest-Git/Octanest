use octanest_api::{build_cors, router};
use octanest_db::Database;
use tracing_subscriber::EnvFilter;

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
    }

    let bind = std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .unwrap_or_else(|e| {
            eprintln!("bind {bind}: {e}");
            std::process::exit(1);
        });
    tracing::info!("octanest-api listening on {bind}");

    let app = router(db, cors);
    axum::serve(listener, app).await.expect("server error");
}
