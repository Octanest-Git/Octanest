//! LFS object / link persistence (D-LFS-02, D-LFS-03).

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct LfsObjectRow {
    pub oid: String,
    pub size: i64,
    pub created_at: String,
}

pub async fn get_lfs_enabled(pool: &DbPool, repo_id: &str) -> Result<bool, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query("SELECT lfs_enabled FROM repositories WHERE id = ?")
                .bind(repo_id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("get_lfs_enabled: {e}"))?;
            Ok(row
                .map(|r| r.try_get::<i64, _>("lfs_enabled").unwrap_or(0) != 0)
                .unwrap_or(false))
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query("SELECT lfs_enabled FROM repositories WHERE id = $1")
                .bind(repo_id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("get_lfs_enabled: {e}"))?;
            Ok(row
                .map(|r| r.try_get::<bool, _>("lfs_enabled").unwrap_or(false))
                .unwrap_or(false))
        }
        DbPool::MySql(p) => {
            let row = sqlx::query("SELECT lfs_enabled FROM repositories WHERE id = ?")
                .bind(repo_id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("get_lfs_enabled: {e}"))?;
            Ok(row
                .map(|r| r.try_get::<i8, _>("lfs_enabled").unwrap_or(0) != 0)
                .unwrap_or(false))
        }
    }
}

pub async fn set_lfs_enabled(pool: &DbPool, repo_id: &str, enabled: bool) -> Result<(), String> {
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query("UPDATE repositories SET lfs_enabled = ? WHERE id = ?")
                .bind(if enabled { 1 } else { 0 })
                .bind(repo_id)
                .execute(p)
                .await
                .map_err(|e| format!("set_lfs_enabled: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query("UPDATE repositories SET lfs_enabled = $1 WHERE id = $2")
                .bind(enabled)
                .bind(repo_id)
                .execute(p)
                .await
                .map_err(|e| format!("set_lfs_enabled: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query("UPDATE repositories SET lfs_enabled = ? WHERE id = ?")
                .bind(if enabled { 1 } else { 0 })
                .bind(repo_id)
                .execute(p)
                .await
                .map_err(|e| format!("set_lfs_enabled: {e}"))?;
        }
    }
    Ok(())
}

pub async fn find_lfs_object(pool: &DbPool, oid: &str) -> Result<Option<LfsObjectRow>, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT oid, size, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at FROM lfs_objects WHERE oid = ?",
            )
            .bind(oid)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find_lfs_object: {e}"))?;
            row.map(|r| {
                Ok(LfsObjectRow {
                    oid: r.try_get("oid").map_err(|e| format!("oid: {e}"))?,
                    size: r.try_get("size").map_err(|e| format!("size: {e}"))?,
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| format!("created_at: {e}"))?,
                })
            })
            .transpose()
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT oid, size, to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at FROM lfs_objects WHERE oid = $1",
            )
            .bind(oid)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find_lfs_object: {e}"))?;
            row.map(|r| {
                Ok(LfsObjectRow {
                    oid: r.try_get("oid").map_err(|e| format!("oid: {e}"))?,
                    size: r.try_get("size").map_err(|e| format!("size: {e}"))?,
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| format!("created_at: {e}"))?,
                })
            })
            .transpose()
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                "SELECT oid, size, DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at FROM lfs_objects WHERE oid = ?",
            )
            .bind(oid)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find_lfs_object: {e}"))?;
            row.map(|r| {
                Ok(LfsObjectRow {
                    oid: r.try_get("oid").map_err(|e| format!("oid: {e}"))?,
                    size: r
                        .try_get::<i64, _>("size")
                        .map_err(|e| format!("size: {e}"))?,
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| format!("created_at: {e}"))?,
                })
            })
            .transpose()
        }
    }
}

pub async fn upsert_lfs_object(pool: &DbPool, oid: &str, size: i64) -> Result<(), String> {
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO lfs_objects (oid, size) VALUES (?, ?) ON CONFLICT(oid) DO NOTHING",
            )
            .bind(oid)
            .bind(size)
            .execute(p)
            .await
            .map_err(|e| format!("upsert_lfs_object: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO lfs_objects (oid, size) VALUES ($1, $2) ON CONFLICT (oid) DO NOTHING",
            )
            .bind(oid)
            .bind(size)
            .execute(p)
            .await
            .map_err(|e| format!("upsert_lfs_object: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO lfs_objects (oid, size) VALUES (?, ?) ON DUPLICATE KEY UPDATE oid = oid",
            )
            .bind(oid)
            .bind(size)
            .execute(p)
            .await
            .map_err(|e| format!("upsert_lfs_object: {e}"))?;
        }
    }
    Ok(())
}

pub async fn link_lfs_object(
    pool: &DbPool,
    repository_id: &str,
    oid: &str,
) -> Result<(), String> {
    link_lfs_object_as(pool, repository_id, oid, None).await
}

pub async fn link_lfs_object_as(
    pool: &DbPool,
    repository_id: &str,
    oid: &str,
    uploaded_by: Option<&str>,
) -> Result<(), String> {
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO lfs_object_links (repository_id, oid, refcount, uploaded_by_user_id) VALUES (?, ?, 1, ?)
                 ON CONFLICT(repository_id, oid) DO UPDATE SET refcount = refcount + 1",
            )
            .bind(repository_id)
            .bind(oid)
            .bind(uploaded_by)
            .execute(p)
            .await
            .map_err(|e| format!("link_lfs_object: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO lfs_object_links (repository_id, oid, refcount, uploaded_by_user_id) VALUES ($1, $2, 1, $3)
                 ON CONFLICT (repository_id, oid) DO UPDATE SET refcount = lfs_object_links.refcount + 1",
            )
            .bind(repository_id)
            .bind(oid)
            .bind(uploaded_by)
            .execute(p)
            .await
            .map_err(|e| format!("link_lfs_object: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO lfs_object_links (repository_id, oid, refcount, uploaded_by_user_id) VALUES (?, ?, 1, ?)
                 ON DUPLICATE KEY UPDATE refcount = refcount + 1",
            )
            .bind(repository_id)
            .bind(oid)
            .bind(uploaded_by)
            .execute(p)
            .await
            .map_err(|e| format!("link_lfs_object: {e}"))?;
        }
    }
    Ok(())
}

pub async fn has_lfs_link(
    pool: &DbPool,
    repository_id: &str,
    oid: &str,
) -> Result<bool, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT 1 AS ok FROM lfs_object_links WHERE repository_id = ? AND oid = ?",
            )
            .bind(repository_id)
            .bind(oid)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("has_lfs_link: {e}"))?;
            Ok(row.is_some())
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT 1 AS ok FROM lfs_object_links WHERE repository_id = $1 AND oid = $2",
            )
            .bind(repository_id)
            .bind(oid)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("has_lfs_link: {e}"))?;
            Ok(row.is_some())
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                "SELECT 1 AS ok FROM lfs_object_links WHERE repository_id = ? AND oid = ?",
            )
            .bind(repository_id)
            .bind(oid)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("has_lfs_link: {e}"))?;
            Ok(row.is_some())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LfsSettingsRow {
    /// NULL = use env default.
    pub max_object_bytes: Option<i64>,
    pub quota_repo_bytes: Option<i64>,
    pub quota_user_bytes: Option<i64>,
}

pub async fn get_lfs_settings(pool: &DbPool) -> Result<LfsSettingsRow, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT max_object_bytes, quota_repo_bytes, quota_user_bytes FROM instance_lfs_settings WHERE id = 1",
            )
            .fetch_one(p)
            .await
            .map_err(|e| format!("get_lfs_settings: {e}"))?;
            Ok(LfsSettingsRow {
                max_object_bytes: row.try_get::<Option<i64>, _>("max_object_bytes").unwrap_or(None),
                quota_repo_bytes: row.try_get::<Option<i64>, _>("quota_repo_bytes").unwrap_or(None),
                quota_user_bytes: row.try_get::<Option<i64>, _>("quota_user_bytes").unwrap_or(None),
            })
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT max_object_bytes, quota_repo_bytes, quota_user_bytes FROM instance_lfs_settings WHERE id = 1",
            )
            .fetch_one(p)
            .await
            .map_err(|e| format!("get_lfs_settings: {e}"))?;
            Ok(LfsSettingsRow {
                max_object_bytes: row.try_get::<Option<i64>, _>("max_object_bytes").unwrap_or(None),
                quota_repo_bytes: row.try_get::<Option<i64>, _>("quota_repo_bytes").unwrap_or(None),
                quota_user_bytes: row.try_get::<Option<i64>, _>("quota_user_bytes").unwrap_or(None),
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                "SELECT max_object_bytes, quota_repo_bytes, quota_user_bytes FROM instance_lfs_settings WHERE id = 1",
            )
            .fetch_one(p)
            .await
            .map_err(|e| format!("get_lfs_settings: {e}"))?;
            Ok(LfsSettingsRow {
                max_object_bytes: row.try_get::<Option<i64>, _>("max_object_bytes").unwrap_or(None),
                quota_repo_bytes: row.try_get::<Option<i64>, _>("quota_repo_bytes").unwrap_or(None),
                quota_user_bytes: row.try_get::<Option<i64>, _>("quota_user_bytes").unwrap_or(None),
            })
        }
    }
}

pub async fn update_lfs_settings(
    pool: &DbPool,
    max_object_bytes: Option<i64>,
    quota_repo_bytes: Option<i64>,
    quota_user_bytes: Option<i64>,
) -> Result<LfsSettingsRow, String> {
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE instance_lfs_settings SET
                   max_object_bytes = ?,
                   quota_repo_bytes = ?,
                   quota_user_bytes = ?,
                   updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
                 WHERE id = 1",
            )
            .bind(max_object_bytes)
            .bind(quota_repo_bytes)
            .bind(quota_user_bytes)
            .execute(p)
            .await
            .map_err(|e| format!("update_lfs_settings: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE instance_lfs_settings SET
                   max_object_bytes = $1,
                   quota_repo_bytes = $2,
                   quota_user_bytes = $3,
                   updated_at = NOW()
                 WHERE id = 1",
            )
            .bind(max_object_bytes)
            .bind(quota_repo_bytes)
            .bind(quota_user_bytes)
            .execute(p)
            .await
            .map_err(|e| format!("update_lfs_settings: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE instance_lfs_settings SET
                   max_object_bytes = ?,
                   quota_repo_bytes = ?,
                   quota_user_bytes = ?,
                   updated_at = CURRENT_TIMESTAMP(3)
                 WHERE id = 1",
            )
            .bind(max_object_bytes)
            .bind(quota_repo_bytes)
            .bind(quota_user_bytes)
            .execute(p)
            .await
            .map_err(|e| format!("update_lfs_settings: {e}"))?;
        }
    }
    get_lfs_settings(pool).await
}

/// Logical bytes charged to a repository (sum of linked object sizes).
pub async fn repo_logical_bytes(pool: &DbPool, repository_id: &str) -> Result<i64, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(SUM(o.size), 0) AS total
                 FROM lfs_object_links l
                 JOIN lfs_objects o ON o.oid = l.oid
                 WHERE l.repository_id = ?",
            )
            .bind(repository_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("repo_logical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(SUM(o.size), 0) AS total
                 FROM lfs_object_links l
                 JOIN lfs_objects o ON o.oid = l.oid
                 WHERE l.repository_id = $1",
            )
            .bind(repository_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("repo_logical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(SUM(o.size), 0) AS total
                 FROM lfs_object_links l
                 JOIN lfs_objects o ON o.oid = l.oid
                 WHERE l.repository_id = ?",
            )
            .bind(repository_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("repo_logical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
    }
}

/// Logical bytes across all repos owned by `owner_id` (user or org id).
pub async fn owner_logical_bytes(pool: &DbPool, owner_id: &str) -> Result<i64, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(SUM(o.size), 0) AS total
                 FROM lfs_object_links l
                 JOIN lfs_objects o ON o.oid = l.oid
                 JOIN repositories r ON r.id = l.repository_id
                 WHERE r.owner_id = ?",
            )
            .bind(owner_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("owner_logical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(SUM(o.size), 0) AS total
                 FROM lfs_object_links l
                 JOIN lfs_objects o ON o.oid = l.oid
                 JOIN repositories r ON r.id = l.repository_id
                 WHERE r.owner_id = $1",
            )
            .bind(owner_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("owner_logical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                "SELECT COALESCE(SUM(o.size), 0) AS total
                 FROM lfs_object_links l
                 JOIN lfs_objects o ON o.oid = l.oid
                 JOIN repositories r ON r.id = l.repository_id
                 WHERE r.owner_id = ?",
            )
            .bind(owner_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("owner_logical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
    }
}

/// Physical unique OID bytes on the instance.
pub async fn physical_bytes(pool: &DbPool) -> Result<i64, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query("SELECT COALESCE(SUM(size), 0) AS total FROM lfs_objects")
                .fetch_one(p)
                .await
                .map_err(|e| format!("physical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query("SELECT COALESCE(SUM(size), 0) AS total FROM lfs_objects")
                .fetch_one(p)
                .await
                .map_err(|e| format!("physical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
        DbPool::MySql(p) => {
            let row = sqlx::query("SELECT COALESCE(SUM(size), 0) AS total FROM lfs_objects")
                .fetch_one(p)
                .await
                .map_err(|e| format!("physical_bytes: {e}"))?;
            Ok(row.try_get::<i64, _>("total").unwrap_or(0))
        }
    }
}
