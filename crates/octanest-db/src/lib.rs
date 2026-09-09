//! Uniform database adapter boundary (dialects proven in Phase 2).

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Clone)]
pub struct Database {
    pool: Option<PgPool>,
}

impl Database {
    pub fn skipped() -> Self {
        Self { pool: None }
    }

    /// Connect using `DATABASE_URL` when set; otherwise run without a pool (`skipped` ping).
    pub async fn from_env() -> Result<Self, String> {
        match std::env::var("DATABASE_URL") {
            Ok(url) if !url.is_empty() => {
                let pool = PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&url)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(Self { pool: Some(pool) })
            }
            _ => Ok(Self::skipped()),
        }
    }

    pub async fn ping(&self) -> &'static str {
        let Some(pool) = &self.pool else {
            return "skipped";
        };
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => "ok",
            Err(_) => "error",
        }
    }
}

pub fn connect_stub() -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_connects() {
        assert!(connect_stub().is_ok());
    }
}
