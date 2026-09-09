//! User CRUD via `DbPool` match — dialect branching stays in this crate.

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub display_name: String,
    pub bio: String,
    pub avatar_path: Option<String>,
    pub is_admin: bool,
    pub email_verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

macro_rules! map_user {
    ($row:expr) => {{
        let row = $row;
        let is_admin = row
            .try_get::<bool, _>("is_admin")
            .or_else(|_| {
                row.try_get::<i64, _>("is_admin")
                    .map(|v| v != 0)
                    .or_else(|_| row.try_get::<i8, _>("is_admin").map(|v| v != 0))
            })
            .map_err(|e| format!("user row: {e}"))?;
        UserRow {
            id: row.try_get("id").map_err(|e| format!("user row: {e}"))?,
            email: row.try_get("email").map_err(|e| format!("user row: {e}"))?,
            username: row.try_get("username").map_err(|e| format!("user row: {e}"))?,
            password_hash: row
                .try_get("password_hash")
                .map_err(|e| format!("user row: {e}"))?,
            display_name: row
                .try_get("display_name")
                .map_err(|e| format!("user row: {e}"))?,
            bio: row.try_get("bio").map_err(|e| format!("user row: {e}"))?,
            avatar_path: row
                .try_get("avatar_path")
                .map_err(|e| format!("user row: {e}"))?,
            is_admin,
            email_verified_at: row
                .try_get("email_verified_at")
                .map_err(|e| format!("user row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("user row: {e}"))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("user row: {e}"))?,
        }
    }};
}

const USER_SELECT_PG: &str = "SELECT id, email, username, password_hash, display_name, bio, avatar_path, is_admin,
       CASE WHEN email_verified_at IS NULL THEN NULL
            ELSE to_char(email_verified_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END AS email_verified_at,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at,
       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS updated_at
FROM users";

const USER_SELECT_MYSQL: &str = "SELECT id, email, username, password_hash, display_name, bio, avatar_path, is_admin,
       CASE WHEN email_verified_at IS NULL THEN NULL
            ELSE DATE_FORMAT(email_verified_at, '%Y-%m-%dT%H:%i:%sZ') END AS email_verified_at,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
       DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
FROM users";

const USER_SELECT_SQLITE: &str = "SELECT id, email, username, password_hash, display_name, bio, avatar_path, is_admin,
       CASE WHEN email_verified_at IS NULL THEN NULL
            ELSE strftime('%Y-%m-%dT%H:%M:%SZ', email_verified_at) END AS email_verified_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
FROM users";

pub async fn insert_user(
    pool: &DbPool,
    id: &str,
    email: &str,
    username: &str,
    password_hash: Option<&str>,
    display_name: &str,
    bio: &str,
    avatar_path: Option<&str>,
    is_admin: bool,
) -> Result<UserRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO users (id, email, username, password_hash, display_name, bio, avatar_path, is_admin)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(id)
            .bind(email)
            .bind(username)
            .bind(password_hash)
            .bind(display_name)
            .bind(bio)
            .bind(avatar_path)
            .bind(is_admin)
            .execute(p)
            .await
            .map_err(|e| format!("insert user failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO users (id, email, username, password_hash, display_name, bio, avatar_path, is_admin)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(email)
            .bind(username)
            .bind(password_hash)
            .bind(display_name)
            .bind(bio)
            .bind(avatar_path)
            .bind(is_admin)
            .execute(p)
            .await
            .map_err(|e| format!("insert user failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO users (id, email, username, password_hash, display_name, bio, avatar_path, is_admin)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(id)
            .bind(email)
            .bind(username)
            .bind(password_hash)
            .bind(display_name)
            .bind(bio)
            .bind(avatar_path)
            .bind(if is_admin { 1 } else { 0 })
            .execute(p)
            .await
            .map_err(|e| format!("insert user failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "insert user failed: row missing after insert".into())
}

pub async fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<UserRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_PG} WHERE email = $1"))
                .bind(email)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by email failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_MYSQL} WHERE email = ?"))
                .bind(email)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by email failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_SQLITE} WHERE email = ?1"))
                .bind(email)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by email failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
    }
}

pub async fn find_by_username(pool: &DbPool, username: &str) -> Result<Option<UserRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_PG} WHERE username = $1"))
                .bind(username)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by username failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_MYSQL} WHERE username = ?"))
                .bind(username)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by username failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_SQLITE} WHERE username = ?1"))
                .bind(username)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by username failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
    }
}

pub async fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<UserRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_PG} WHERE id = $1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_MYSQL} WHERE id = ?"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{USER_SELECT_SQLITE} WHERE id = ?1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find user by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_user!(&r)),
                None => None,
            })
        }
    }
}

pub async fn update_profile(
    pool: &DbPool,
    id: &str,
    display_name: &str,
    username: &str,
    bio: &str,
    avatar_path: Option<&str>,
) -> Result<UserRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE users SET display_name = $2, username = $3, bio = $4, avatar_path = $5, updated_at = now()
WHERE id = $1",
            )
            .bind(id)
            .bind(display_name)
            .bind(username)
            .bind(bio)
            .bind(avatar_path)
            .execute(p)
            .await
            .map_err(|e| format!("update user profile failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE users SET display_name = ?, username = ?, bio = ?, avatar_path = ?, updated_at = NOW()
WHERE id = ?",
            )
            .bind(display_name)
            .bind(username)
            .bind(bio)
            .bind(avatar_path)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("update user profile failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE users SET display_name = ?2, username = ?3, bio = ?4, avatar_path = ?5,
    updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1",
            )
            .bind(id)
            .bind(display_name)
            .bind(username)
            .bind(bio)
            .bind(avatar_path)
            .execute(p)
            .await
            .map_err(|e| format!("update user profile failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "update user profile failed: user not found".into())
}

pub async fn count_users(pool: &DbPool) -> Result<i64, String> {
    match pool {
        DbPool::Postgres(p) => sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users")
            .fetch_one(p)
            .await
            .map_err(|e| format!("count users failed: {e}")),
        DbPool::MySql(p) => sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users")
            .fetch_one(p)
            .await
            .map_err(|e| format!("count users failed: {e}")),
        DbPool::Sqlite(p) => sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users")
            .fetch_one(p)
            .await
            .map_err(|e| format!("count users failed: {e}")),
    }
}
