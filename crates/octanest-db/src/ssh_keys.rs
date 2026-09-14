//! SSH public key CRUD via `DbPool` match — fingerprint UNIQUE (D-SSH-05).
//! Revoke is hard-delete (ASSUME A3); no `revoked_at` column.

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct SshKeyRow {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub public_key: String,
    pub fingerprint: String,
    pub key_type: String,
    pub last_used_at: Option<String>,
    pub last_used_ip: Option<String>,
    pub created_at: String,
}

macro_rules! map_ssh_key {
    ($row:expr) => {{
        let row = $row;
        SshKeyRow {
            id: row.try_get("id").map_err(|e| format!("ssh key row: {e}"))?,
            user_id: row
                .try_get("user_id")
                .map_err(|e| format!("ssh key row: {e}"))?,
            title: row
                .try_get("title")
                .map_err(|e| format!("ssh key row: {e}"))?,
            public_key: row
                .try_get("public_key")
                .map_err(|e| format!("ssh key row: {e}"))?,
            fingerprint: row
                .try_get("fingerprint")
                .map_err(|e| format!("ssh key row: {e}"))?,
            key_type: row
                .try_get("key_type")
                .map_err(|e| format!("ssh key row: {e}"))?,
            last_used_at: row
                .try_get("last_used_at")
                .map_err(|e| format!("ssh key row: {e}"))?,
            last_used_ip: row
                .try_get("last_used_ip")
                .map_err(|e| format!("ssh key row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("ssh key row: {e}"))?,
        }
    }};
}

const SSH_SELECT_PG: &str = "SELECT id, user_id, title, public_key, fingerprint, key_type,
       CASE WHEN last_used_at IS NULL THEN NULL
            ELSE to_char(last_used_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END AS last_used_at,
       last_used_ip,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
FROM ssh_public_keys";

const SSH_SELECT_MYSQL: &str = "SELECT id, user_id, title, public_key, fingerprint, key_type,
       CASE WHEN last_used_at IS NULL THEN NULL
            ELSE DATE_FORMAT(last_used_at, '%Y-%m-%dT%H:%i:%sZ') END AS last_used_at,
       last_used_ip,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at
FROM ssh_public_keys";

const SSH_SELECT_SQLITE: &str = "SELECT id, user_id, title, public_key, fingerprint, key_type,
       CASE WHEN last_used_at IS NULL THEN NULL
            ELSE strftime('%Y-%m-%dT%H:%M:%SZ', last_used_at) END AS last_used_at,
       last_used_ip,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at
FROM ssh_public_keys";

/// Insert a registered SSH public key row.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &DbPool,
    id: &str,
    user_id: &str,
    title: &str,
    public_key: &str,
    fingerprint: &str,
    key_type: &str,
) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO ssh_public_keys
(id, user_id, title, public_key, fingerprint, key_type)
VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(id)
            .bind(user_id)
            .bind(title)
            .bind(public_key)
            .bind(fingerprint)
            .bind(key_type)
            .execute(p)
            .await
            .map_err(|e| format!("create ssh key failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO ssh_public_keys
(id, user_id, title, public_key, fingerprint, key_type)
VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(user_id)
            .bind(title)
            .bind(public_key)
            .bind(fingerprint)
            .bind(key_type)
            .execute(p)
            .await
            .map_err(|e| format!("create ssh key failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO ssh_public_keys
(id, user_id, title, public_key, fingerprint, key_type)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(id)
            .bind(user_id)
            .bind(title)
            .bind(public_key)
            .bind(fingerprint)
            .bind(key_type)
            .execute(p)
            .await
            .map_err(|e| format!("create ssh key failed: {e}"))?;
        }
    }
    Ok(())
}

/// Lookup by OpenSSH SHA256 fingerprint (`SHA256:…`).
pub async fn find_by_fingerprint(
    pool: &DbPool,
    fingerprint: &str,
) -> Result<Option<SshKeyRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{SSH_SELECT_PG} WHERE fingerprint = $1"))
                .bind(fingerprint)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find ssh key failed: {e}"))?;
            match row {
                Some(r) => Ok(Some(map_ssh_key!(&r))),
                None => Ok(None),
            }
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{SSH_SELECT_MYSQL} WHERE fingerprint = ?"))
                .bind(fingerprint)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find ssh key failed: {e}"))?;
            match row {
                Some(r) => Ok(Some(map_ssh_key!(&r))),
                None => Ok(None),
            }
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{SSH_SELECT_SQLITE} WHERE fingerprint = ?1"))
                .bind(fingerprint)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find ssh key failed: {e}"))?;
            match row {
                Some(r) => Ok(Some(map_ssh_key!(&r))),
                None => Ok(None),
            }
        }
    }
}

/// Keys for a user, newest first (`created_at DESC`).
pub async fn list_for_user(pool: &DbPool, user_id: &str) -> Result<Vec<SshKeyRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let rows = sqlx::query(&format!(
                "{SSH_SELECT_PG} WHERE user_id = $1 ORDER BY created_at DESC, id DESC"
            ))
            .bind(user_id)
            .fetch_all(p)
            .await
            .map_err(|e| format!("list ssh keys failed: {e}"))?;
            let mut mapped = Vec::with_capacity(rows.len());
            for r in rows {
                mapped.push(map_ssh_key!(&r));
            }
            Ok(mapped)
        }
        DbPool::MySql(p) => {
            let rows = sqlx::query(&format!(
                "{SSH_SELECT_MYSQL} WHERE user_id = ? ORDER BY created_at DESC, id DESC"
            ))
            .bind(user_id)
            .fetch_all(p)
            .await
            .map_err(|e| format!("list ssh keys failed: {e}"))?;
            let mut mapped = Vec::with_capacity(rows.len());
            for r in rows {
                mapped.push(map_ssh_key!(&r));
            }
            Ok(mapped)
        }
        DbPool::Sqlite(p) => {
            let rows = sqlx::query(&format!(
                "{SSH_SELECT_SQLITE} WHERE user_id = ?1 ORDER BY created_at DESC, id DESC"
            ))
            .bind(user_id)
            .fetch_all(p)
            .await
            .map_err(|e| format!("list ssh keys failed: {e}"))?;
            let mut mapped = Vec::with_capacity(rows.len());
            for r in rows {
                mapped.push(map_ssh_key!(&r));
            }
            Ok(mapped)
        }
    }
}

/// Hard-delete a key by id. Idempotent when missing.
pub async fn revoke(pool: &DbPool, id: &str) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query("DELETE FROM ssh_public_keys WHERE id = $1")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("revoke ssh key failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query("DELETE FROM ssh_public_keys WHERE id = ?")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("revoke ssh key failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query("DELETE FROM ssh_public_keys WHERE id = ?1")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("revoke ssh key failed: {e}"))?;
        }
    }
    Ok(())
}

/// Update last-used timestamp and optional client IP after successful SSH auth.
pub async fn touch_last_used(
    pool: &DbPool,
    id: &str,
    last_used_at: &str,
    last_used_ip: Option<&str>,
) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE ssh_public_keys
SET last_used_at = $2::timestamptz, last_used_ip = $3
WHERE id = $1",
            )
            .bind(id)
            .bind(last_used_at)
            .bind(last_used_ip)
            .execute(p)
            .await
            .map_err(|e| format!("touch ssh key failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE ssh_public_keys SET last_used_at = ?, last_used_ip = ?
WHERE id = ?",
            )
            .bind(last_used_at)
            .bind(last_used_ip)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("touch ssh key failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE ssh_public_keys SET last_used_at = ?2, last_used_ip = ?3
WHERE id = ?1",
            )
            .bind(id)
            .bind(last_used_at)
            .bind(last_used_ip)
            .execute(p)
            .await
            .map_err(|e| format!("touch ssh key failed: {e}"))?;
        }
    }
    Ok(())
}
