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
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO lfs_object_links (repository_id, oid, refcount) VALUES (?, ?, 1)
                 ON CONFLICT(repository_id, oid) DO UPDATE SET refcount = refcount + 1",
            )
            .bind(repository_id)
            .bind(oid)
            .execute(p)
            .await
            .map_err(|e| format!("link_lfs_object: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO lfs_object_links (repository_id, oid, refcount) VALUES ($1, $2, 1)
                 ON CONFLICT (repository_id, oid) DO UPDATE SET refcount = lfs_object_links.refcount + 1",
            )
            .bind(repository_id)
            .bind(oid)
            .execute(p)
            .await
            .map_err(|e| format!("link_lfs_object: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO lfs_object_links (repository_id, oid, refcount) VALUES (?, ?, 1)
                 ON DUPLICATE KEY UPDATE refcount = refcount + 1",
            )
            .bind(repository_id)
            .bind(oid)
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
