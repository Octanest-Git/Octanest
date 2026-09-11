//! User CRUD via `DbPool` match — dialect branching stays in this crate.

use octanest_core::Role;
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
    pub role: Role,
    pub must_change_credentials: bool,
    pub email_verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

macro_rules! map_user {
    ($row:expr) => {{
        let row = $row;
        let role_raw: String = row
            .try_get("role")
            .map_err(|e| format!("user row: {e}"))?;
        let role = Role::parse(&role_raw).map_err(|e| format!("user row: {e}"))?;
        let must_change_credentials = row
            .try_get::<bool, _>("must_change_credentials")
            .or_else(|_| {
                row.try_get::<i64, _>("must_change_credentials")
                    .map(|v| v != 0)
                    .or_else(|_| {
                        row.try_get::<i8, _>("must_change_credentials")
                            .map(|v| v != 0)
                    })
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
            role,
            must_change_credentials,
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

const USER_SELECT_PG: &str = "SELECT id, email, username, password_hash, display_name, bio, avatar_path, role, must_change_credentials,
       CASE WHEN email_verified_at IS NULL THEN NULL
            ELSE to_char(email_verified_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END AS email_verified_at,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at,
       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS updated_at
FROM users";

const USER_SELECT_MYSQL: &str = "SELECT id, email, username, password_hash, display_name, bio, avatar_path, role, must_change_credentials,
       CASE WHEN email_verified_at IS NULL THEN NULL
            ELSE DATE_FORMAT(email_verified_at, '%Y-%m-%dT%H:%i:%sZ') END AS email_verified_at,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
       DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
FROM users";

const USER_SELECT_SQLITE: &str = "SELECT id, email, username, password_hash, display_name, bio, avatar_path, role, must_change_credentials,
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
    role: Role,
) -> Result<UserRow, String> {
    let role_s = role.as_str();
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO users (id, email, username, password_hash, display_name, bio, avatar_path, role)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(id)
            .bind(email)
            .bind(username)
            .bind(password_hash)
            .bind(display_name)
            .bind(bio)
            .bind(avatar_path)
            .bind(role_s)
            .execute(p)
            .await
            .map_err(|e| format!("insert user failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO users (id, email, username, password_hash, display_name, bio, avatar_path, role)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(email)
            .bind(username)
            .bind(password_hash)
            .bind(display_name)
            .bind(bio)
            .bind(avatar_path)
            .bind(role_s)
            .execute(p)
            .await
            .map_err(|e| format!("insert user failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO users (id, email, username, password_hash, display_name, bio, avatar_path, role)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(id)
            .bind(email)
            .bind(username)
            .bind(password_hash)
            .bind(display_name)
            .bind(bio)
            .bind(avatar_path)
            .bind(role_s)
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

/// Count users with `role = sys-admin`.
pub async fn count_sys_admins(pool: &DbPool) -> Result<i64, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE role = 'sys-admin'")
                .fetch_one(p)
                .await
                .map_err(|e| format!("count sys-admins failed: {e}"))
        }
        DbPool::MySql(p) => {
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE role = 'sys-admin'")
                .fetch_one(p)
                .await
                .map_err(|e| format!("count sys-admins failed: {e}"))
        }
        DbPool::Sqlite(p) => {
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE role = 'sys-admin'")
                .fetch_one(p)
                .await
                .map_err(|e| format!("count sys-admins failed: {e}"))
        }
    }
}

/// Set `email_verified_at` to the given RFC3339 / dialect timestamp string.
pub async fn set_email_verified_at(
    pool: &DbPool,
    id: &str,
    at: &str,
) -> Result<UserRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE users SET email_verified_at = $2::timestamptz, updated_at = now()
WHERE id = $1",
            )
            .bind(id)
            .bind(at)
            .execute(p)
            .await
            .map_err(|e| format!("set email_verified_at failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE users SET email_verified_at = ?, updated_at = NOW() WHERE id = ?",
            )
            .bind(at)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("set email_verified_at failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE users SET email_verified_at = ?2,
    updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1",
            )
            .bind(id)
            .bind(at)
            .execute(p)
            .await
            .map_err(|e| format!("set email_verified_at failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "set email_verified_at failed: user not found".into())
}

/// Clear `email_verified_at` (e.g. email change — D-05).
pub async fn clear_email_verified_at(pool: &DbPool, id: &str) -> Result<UserRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE users SET email_verified_at = NULL, updated_at = now() WHERE id = $1",
            )
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("clear email_verified_at failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE users SET email_verified_at = NULL, updated_at = NOW() WHERE id = ?",
            )
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("clear email_verified_at failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE users SET email_verified_at = NULL,
    updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1",
            )
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("clear email_verified_at failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "clear email_verified_at failed: user not found".into())
}

/// Replace the user's password hash (password reset / change).
pub async fn set_password_hash(
    pool: &DbPool,
    id: &str,
    password_hash: &str,
) -> Result<UserRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE users SET password_hash = $2, updated_at = now() WHERE id = $1",
            )
            .bind(id)
            .bind(password_hash)
            .execute(p)
            .await
            .map_err(|e| format!("set password_hash failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE users SET password_hash = ?, updated_at = NOW() WHERE id = ?",
            )
            .bind(password_hash)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("set password_hash failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE users SET password_hash = ?2,
    updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1",
            )
            .bind(id)
            .bind(password_hash)
            .execute(p)
            .await
            .map_err(|e| format!("set password_hash failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "set password_hash failed: user not found".into())
}

/// Update the user's email (forced credential confirm / profile change).
pub async fn update_user_email(pool: &DbPool, id: &str, email: &str) -> Result<UserRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query("UPDATE users SET email = $2, updated_at = now() WHERE id = $1")
                .bind(id)
                .bind(email)
                .execute(p)
                .await
                .map_err(|e| format!("update user email failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query("UPDATE users SET email = ?, updated_at = NOW() WHERE id = ?")
                .bind(email)
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("update user email failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE users SET email = ?2,
    updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1",
            )
            .bind(id)
            .bind(email)
            .execute(p)
            .await
            .map_err(|e| format!("update user email failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "update user email failed: user not found".into())
}

/// Set or clear `must_change_credentials` (ENV seed sets true; confirm clears).
pub async fn set_must_change_credentials(
    pool: &DbPool,
    id: &str,
    must_change: bool,
) -> Result<UserRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE users SET must_change_credentials = $2, updated_at = now() WHERE id = $1",
            )
            .bind(id)
            .bind(must_change)
            .execute(p)
            .await
            .map_err(|e| format!("set must_change_credentials failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE users SET must_change_credentials = ?, updated_at = NOW() WHERE id = ?",
            )
            .bind(must_change)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("set must_change_credentials failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE users SET must_change_credentials = ?2,
    updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1",
            )
            .bind(id)
            .bind(must_change)
            .execute(p)
            .await
            .map_err(|e| format!("set must_change_credentials failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "set must_change_credentials failed: user not found".into())
}

/// Clear the forced-credentials flag after successful confirm.
pub async fn clear_must_change_credentials(pool: &DbPool, id: &str) -> Result<UserRow, String> {
    set_must_change_credentials(pool, id, false).await
}
