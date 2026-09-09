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

    let db = match Database::from_env().await {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!("database connect failed, continuing without pool: {e}");
            Database::skipped()
        }
    };

    let bind = std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap_or_else(|e| {
        eprintln!("bind {bind}: {e}");
        std::process::exit(1);
    });
    tracing::info!("octanest-api listening on {bind}");

    let app = router(db, cors);
    axum::serve(listener, app).await.expect("server error");
}
