//! Explicit migrate CLI for `OCTANEST_AUTO_MIGRATE=false` and `make db-switch-dialect` (D-07, D-16).
//!
//! `cargo run -p octanest-db --bin migrate -- [--assert-empty] [--force-empty]`

use octanest_db::{redact_url, Database};

#[tokio::main]
async fn main() {
    let mut assert_empty = false;
    let mut force_empty = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--assert-empty" => assert_empty = true,
            "--force-empty" => force_empty = true,
            other => {
                eprintln!("usage: migrate [--assert-empty] [--force-empty]: unknown flag {other}");
                std::process::exit(2);
            }
        }
    }

    let url = match std::env::var("DATABASE_URL") {
        Ok(url) if !url.is_empty() => url,
        _ => {
            eprintln!("DATABASE_URL is not set");
            std::process::exit(1);
        }
    };

    eprintln!("target: {}", redact_url(&url));

    let db = match Database::connect(&url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("connect failed: {e}");
            std::process::exit(1);
        }
    };

    if assert_empty && !force_empty {
        match db.is_empty().await {
            Ok(true) => {}
            Ok(false) => {
                eprintln!(
                    "refusing to migrate: target database is not empty (pass --force-empty to override)"
                );
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("is_empty check failed: {e}");
                std::process::exit(1);
            }
        }
    }

    match db.migrate().await {
        Ok(()) => {
            let dialect = db.dialect().map(|d| d.as_str()).unwrap_or("unknown");
            println!("migrated: {dialect}");
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
