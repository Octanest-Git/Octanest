//! Uniform database adapter boundary — the only place dialect branching is allowed (D-08).

pub mod auth_identities;
pub mod auth_settings;
pub mod dialect;
pub mod email_tokens;
pub mod migrate;
pub mod pool;
pub mod probe;
pub mod repositories;
pub mod sessions;
pub mod users;

pub use dialect::{redact_url, resolve_dialect, resolve_dialect_from_env, Dialect};
pub use octanest_core::DbProbeResponse;
pub use pool::DbPool;
pub use repositories::RepositoryRow;
pub use users::UserRow;
pub use auth_settings::AuthSettingsRow;

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

    // --- repositories ---

    pub async fn insert_repository(
        &self,
        id: &str,
        owner_id: &str,
        name: &str,
        visibility: &str,
        description: &str,
        default_branch: &str,
    ) -> Result<RepositoryRow, String> {
        repositories::insert_repository(
            self.require_pool()?,
            id,
            owner_id,
            name,
            visibility,
            description,
            default_branch,
        )
        .await
    }

    pub async fn find_repository_by_owner_name(
        &self,
        owner_id: &str,
        name: &str,
    ) -> Result<Option<RepositoryRow>, String> {
        repositories::find_by_owner_and_name(self.require_pool()?, owner_id, name).await
    }

    pub async fn find_repository_by_id(&self, id: &str) -> Result<Option<RepositoryRow>, String> {
        repositories::find_by_id(self.require_pool()?, id).await
    }

    pub async fn list_repositories_by_owner(
        &self,
        owner_id: &str,
    ) -> Result<Vec<RepositoryRow>, String> {
        repositories::list_by_owner(self.require_pool()?, owner_id).await
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
        role: octanest_core::Role,
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
            role,
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

    pub async fn count_sys_admins(&self) -> Result<i64, String> {
        users::count_sys_admins(self.require_pool()?).await
    }

    pub async fn set_email_verified_at(
        &self,
        id: &str,
        at: &str,
    ) -> Result<UserRow, String> {
        users::set_email_verified_at(self.require_pool()?, id, at).await
    }

    pub async fn clear_email_verified_at(&self, id: &str) -> Result<UserRow, String> {
        users::clear_email_verified_at(self.require_pool()?, id).await
    }

    pub async fn set_password_hash(
        &self,
        id: &str,
        password_hash: &str,
    ) -> Result<UserRow, String> {
        users::set_password_hash(self.require_pool()?, id, password_hash).await
    }

    pub async fn update_user_email(&self, id: &str, email: &str) -> Result<UserRow, String> {
        users::update_user_email(self.require_pool()?, id, email).await
    }

    pub async fn set_must_change_credentials(
        &self,
        id: &str,
        must_change: bool,
    ) -> Result<UserRow, String> {
        users::set_must_change_credentials(self.require_pool()?, id, must_change).await
    }

    pub async fn clear_must_change_credentials(&self, id: &str) -> Result<UserRow, String> {
        users::clear_must_change_credentials(self.require_pool()?, id).await
    }

    // --- email tokens ---

    pub async fn upsert_email_token(
        &self,
        id: &str,
        user_id: &str,
        purpose: &str,
        token_hash: &str,
        otp_hash: &str,
        expires_at: &str,
        issue_count: i32,
    ) -> Result<email_tokens::EmailTokenRow, String> {
        email_tokens::upsert_by_user_purpose(
            self.require_pool()?,
            id,
            user_id,
            purpose,
            token_hash,
            otp_hash,
            expires_at,
            issue_count,
        )
        .await
    }

    pub async fn find_email_token_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<email_tokens::EmailTokenRow>, String> {
        email_tokens::find_by_token_hash(self.require_pool()?, token_hash).await
    }

    pub async fn find_email_token_by_otp_hash(
        &self,
        otp_hash: &str,
    ) -> Result<Option<email_tokens::EmailTokenRow>, String> {
        email_tokens::find_by_otp_hash(self.require_pool()?, otp_hash).await
    }

    pub async fn find_email_token_by_user_purpose(
        &self,
        user_id: &str,
        purpose: &str,
    ) -> Result<Option<email_tokens::EmailTokenRow>, String> {
        email_tokens::find_by_user_purpose(self.require_pool()?, user_id, purpose).await
    }

    pub async fn increment_email_token_attempts(&self, id: &str) -> Result<i32, String> {
        email_tokens::increment_attempts(self.require_pool()?, id).await
    }

    pub async fn delete_email_token(&self, id: &str) -> Result<(), String> {
        email_tokens::delete(self.require_pool()?, id).await
    }

    /// Test helper: rewrite token `created_at` for rate-limit simulations.
    pub async fn set_email_token_created_at(
        &self,
        user_id: &str,
        purpose: &str,
        created_at: &str,
    ) -> Result<(), String> {
        email_tokens::set_created_at(self.require_pool()?, user_id, purpose, created_at).await
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
        allow_signup: bool,
    ) -> Result<auth_settings::AuthSettingsRow, String> {
        auth_settings::update(
            self.require_pool()?,
            provider_mode,
            email_provider,
            from_address,
            oidc_issuer,
            oidc_client_id,
            workos_client_id,
            allow_signup,
        )
        .await
    }

    /// Wipe auth data so the instance returns to empty-setup (`needs_setup`).
    /// Deletes sessions, identities, email tokens, and users; resets auth settings
    /// to local/log defaults with signup closed.
    pub async fn factory_reset_instance(&self) -> Result<(), String> {
        let pool = self.require_pool()?;
        match pool {
            Pool::Postgres(p) => {
                sqlx::query("DELETE FROM auth_email_tokens")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM sessions")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM auth_identities")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM users")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query(
                    "UPDATE instance_auth_settings SET
                      provider_mode = 'local',
                      email_provider = 'log',
                      from_address = NULL,
                      oidc_issuer = NULL,
                      oidc_client_id = NULL,
                      workos_client_id = NULL,
                      allow_signup = false,
                      updated_at = now()
                     WHERE id = 1",
                )
                .execute(p)
                .await
                .map_err(|e| e.to_string())?;
            }
            Pool::MySql(p) => {
                sqlx::query("DELETE FROM auth_email_tokens")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM sessions")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM auth_identities")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM users")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query(
                    "UPDATE instance_auth_settings SET
                      provider_mode = 'local',
                      email_provider = 'log',
                      from_address = NULL,
                      oidc_issuer = NULL,
                      oidc_client_id = NULL,
                      workos_client_id = NULL,
                      allow_signup = 0,
                      updated_at = NOW()
                     WHERE id = 1",
                )
                .execute(p)
                .await
                .map_err(|e| e.to_string())?;
            }
            Pool::Sqlite(p) => {
                // Ensure FK cascades / order are honored.
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM auth_email_tokens")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM sessions")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM auth_identities")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM users")
                    .execute(p)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query(
                    "UPDATE instance_auth_settings SET
                      provider_mode = 'local',
                      email_provider = 'log',
                      from_address = NULL,
                      oidc_issuer = NULL,
                      oidc_client_id = NULL,
                      workos_client_id = NULL,
                      allow_signup = 0,
                      updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
                     WHERE id = 1",
                )
                .execute(p)
                .await
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
