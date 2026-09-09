//! Uniform database adapter boundary — the only place dialect branching is allowed (D-08).

pub mod auth_identities;
pub mod auth_settings;
pub mod dialect;
pub mod migrate;
pub mod pool;
pub mod probe;
pub mod sessions;
pub mod users;

pub use dialect::{redact_url, resolve_dialect, resolve_dialect_from_env, Dialect};
pub use octanest_core::DbProbeResponse;
pub use pool::DbPool;
pub use users::UserRow;

use dialect::resolve_dialect_from_env as resolve_from_env;
use pool::DbPool as Pool;

#[derive(Clone)]
pub struct Database {
    pool: Option<Pool>,
    dialect: Option<Dialect>,
}

impl Database {
    pub fn skipped() -> Self {
        Self {
            pool: None,
            dialect: None,
        }
    }

    /// Connect using `DATABASE_URL` when set; otherwise run without a pool (`skipped` ping).
    pub async fn from_env() -> Result<Self, String> {
        match std::env::var("DATABASE_URL") {
            Ok(url) if !url.is_empty() => Self::connect(&url).await,
            _ => Ok(Self::skipped()),
        }
    }

    pub async fn connect(url: &str) -> Result<Self, String> {
        let dialect = resolve_from_env(url)?;
        let pool = Pool::connect(url, dialect).await?;
        Ok(Self {
            pool: Some(pool),
            dialect: Some(dialect),
        })
    }

    pub fn dialect(&self) -> Option<Dialect> {
        self.dialect
    }

    fn require_pool(&self) -> Result<&Pool, String> {
        self.pool
            .as_ref()
            .ok_or_else(|| "database not configured".into())
    }

    pub async fn ping(&self) -> &'static str {
        let Some(pool) = &self.pool else {
            return "skipped";
        };
        let ok = match pool {
            Pool::Postgres(p) => sqlx::query("SELECT 1").execute(p).await.is_ok(),
            Pool::MySql(p) => sqlx::query("SELECT 1").execute(p).await.is_ok(),
            Pool::Sqlite(p) => sqlx::query("SELECT 1").execute(p).await.is_ok(),
        };
        if ok { "ok" } else { "error" }
    }

    pub async fn migrate(&self) -> Result<(), String> {
        let Some(pool) = &self.pool else {
            return Err("database not configured".into());
        };
        migrate::run_migrations(pool).await
    }

    pub async fn is_empty(&self) -> Result<bool, String> {
        let Some(pool) = &self.pool else {
            return Err("database not configured".into());
        };
        migrate::is_empty(pool).await
    }

    pub async fn probe(&self) -> Result<DbProbeResponse, String> {
        let (Some(pool), Some(dialect)) = (&self.pool, self.dialect) else {
            return Err("database not configured".into());
        };
        probe::probe(pool, dialect).await
    }

    // --- users ---

    pub async fn create_user(
        &self,
        id: &str,
        email: &str,
        username: &str,
        password_hash: Option<&str>,
        display_name: &str,
        bio: &str,
        avatar_path: Option<&str>,
        is_admin: bool,
    ) -> Result<UserRow, String> {
        let pool = self.require_pool()?;
        users::insert_user(
            pool,
            id,
            email,
            username,
            password_hash,
            display_name,
            bio,
            avatar_path,
            is_admin,
        )
        .await
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRow>, String> {
        users::find_by_email(self.require_pool()?, email).await
    }

    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<UserRow>, String> {
        users::find_by_username(self.require_pool()?, username).await
    }

    pub async fn find_user_by_id(&self, id: &str) -> Result<Option<UserRow>, String> {
        users::find_by_id(self.require_pool()?, id).await
    }

    pub async fn update_user_profile(
        &self,
        id: &str,
        display_name: &str,
        username: &str,
        bio: &str,
        avatar_path: Option<&str>,
    ) -> Result<UserRow, String> {
        users::update_profile(
            self.require_pool()?,
            id,
            display_name,
            username,
            bio,
            avatar_path,
        )
        .await
    }

    pub async fn count_users(&self) -> Result<i64, String> {
        users::count_users(self.require_pool()?).await
    }

    // --- sessions ---

    pub async fn create_session(
        &self,
        id: &str,
        user_id: &str,
        token_hash: &str,
        expires_at: &str,
        remember_me: bool,
    ) -> Result<(), String> {
        sessions::create(
            self.require_pool()?,
            id,
            user_id,
            token_hash,
            expires_at,
            remember_me,
        )
        .await
    }

    pub async fn find_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<sessions::SessionRow>, String> {
        sessions::find_by_token_hash(self.require_pool()?, token_hash).await
    }

    pub async fn touch_session(
        &self,
        id: &str,
        expires_at: &str,
        last_seen_at: &str,
    ) -> Result<(), String> {
        sessions::touch(self.require_pool()?, id, expires_at, last_seen_at).await
    }

    pub async fn delete_session(&self, id: &str) -> Result<(), String> {
        sessions::delete(self.require_pool()?, id).await
    }

    pub async fn delete_sessions_for_user(&self, user_id: &str) -> Result<u64, String> {
        sessions::delete_all_for_user(self.require_pool()?, user_id).await
    }

    // --- auth identities ---

    pub async fn upsert_auth_identity(
        &self,
        id: &str,
        user_id: &str,
        provider: &str,
        provider_subject: &str,
        provider_email: Option<&str>,
    ) -> Result<auth_identities::AuthIdentityRow, String> {
        auth_identities::upsert(
            self.require_pool()?,
            id,
            user_id,
            provider,
            provider_subject,
            provider_email,
        )
        .await
    }

    pub async fn find_auth_identity(
        &self,
        provider: &str,
        provider_subject: &str,
    ) -> Result<Option<auth_identities::AuthIdentityRow>, String> {
        auth_identities::find(self.require_pool()?, provider, provider_subject).await
    }

    // --- auth settings ---

    pub async fn get_auth_settings(&self) -> Result<auth_settings::AuthSettingsRow, String> {
        auth_settings::get(self.require_pool()?).await
    }

    pub async fn update_auth_settings(
        &self,
        provider_mode: &str,
        email_provider: &str,
        from_address: Option<&str>,
        oidc_issuer: Option<&str>,
        oidc_client_id: Option<&str>,
        workos_client_id: Option<&str>,
    ) -> Result<auth_settings::AuthSettingsRow, String> {
        auth_settings::update(
            self.require_pool()?,
            provider_mode,
            email_provider,
            from_address,
            oidc_issuer,
            oidc_client_id,
            workos_client_id,
        )
        .await
    }
}
