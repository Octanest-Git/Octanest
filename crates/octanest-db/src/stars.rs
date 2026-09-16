//! Repository stars + fork network helpers (Phase 21 / D-SOC-02, D-SOC-14).

use sqlx::Row;

use crate::pool::DbPool;
use crate::repositories::{self, RepositoryRow};

/// Idempotent star: insert membership and bump counter when newly inserted.
pub async fn star_repository(
    pool: &DbPool,
    user_id: &str,
    repository_id: &str,
) -> Result<i64, String> {
    match pool {
        DbPool::Postgres(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("star begin: {e}"))?;
            let inserted = sqlx::query(
                "INSERT INTO repository_stars (user_id, repository_id)
                 VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(repository_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("star insert: {e}"))?
            .rows_affected();
            if inserted > 0 {
                sqlx::query(
                    "UPDATE repositories SET star_count = star_count + 1, updated_at = now()
                     WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(repository_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("star bump: {e}"))?;
            }
            let count: i64 = sqlx::query_scalar(
                "SELECT star_count FROM repositories WHERE id = $1",
            )
            .bind(repository_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("star count: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("star commit: {e}"))?;
            Ok(count)
        }
        DbPool::MySql(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("star begin: {e}"))?;
            let inserted = sqlx::query(
                "INSERT IGNORE INTO repository_stars (user_id, repository_id) VALUES (?, ?)",
            )
            .bind(user_id)
            .bind(repository_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("star insert: {e}"))?
            .rows_affected();
            if inserted > 0 {
                sqlx::query(
                    "UPDATE repositories SET star_count = star_count + 1, updated_at = NOW()
                     WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(repository_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("star bump: {e}"))?;
            }
            let count: i64 = sqlx::query_scalar("SELECT star_count FROM repositories WHERE id = ?")
                .bind(repository_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| format!("star count: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("star commit: {e}"))?;
            Ok(count)
        }
        DbPool::Sqlite(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("star begin: {e}"))?;
            let inserted = sqlx::query(
                "INSERT OR IGNORE INTO repository_stars (user_id, repository_id)
                 VALUES (?1, ?2)",
            )
            .bind(user_id)
            .bind(repository_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("star insert: {e}"))?
            .rows_affected();
            if inserted > 0 {
                sqlx::query(
                    "UPDATE repositories SET star_count = star_count + 1,
                     updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
                     WHERE id = ?1 AND deleted_at IS NULL",
                )
                .bind(repository_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("star bump: {e}"))?;
            }
            let count: i64 = sqlx::query_scalar(
                "SELECT star_count FROM repositories WHERE id = ?1",
            )
            .bind(repository_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("star count: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("star commit: {e}"))?;
            Ok(count)
        }
    }
}

/// Idempotent unstar: delete membership and decrement counter when a row was removed.
pub async fn unstar_repository(
    pool: &DbPool,
    user_id: &str,
    repository_id: &str,
) -> Result<i64, String> {
    match pool {
        DbPool::Postgres(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("unstar begin: {e}"))?;
            let deleted = sqlx::query(
                "DELETE FROM repository_stars WHERE user_id = $1 AND repository_id = $2",
            )
            .bind(user_id)
            .bind(repository_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("unstar delete: {e}"))?
            .rows_affected();
            if deleted > 0 {
                sqlx::query(
                    "UPDATE repositories SET star_count = GREATEST(star_count - 1, 0),
                     updated_at = now() WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(repository_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("unstar bump: {e}"))?;
            }
            let count: i64 = sqlx::query_scalar(
                "SELECT star_count FROM repositories WHERE id = $1",
            )
            .bind(repository_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("unstar count: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("unstar commit: {e}"))?;
            Ok(count)
        }
        DbPool::MySql(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("unstar begin: {e}"))?;
            let deleted = sqlx::query(
                "DELETE FROM repository_stars WHERE user_id = ? AND repository_id = ?",
            )
            .bind(user_id)
            .bind(repository_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("unstar delete: {e}"))?
            .rows_affected();
            if deleted > 0 {
                sqlx::query(
                    "UPDATE repositories SET star_count = GREATEST(star_count - 1, 0),
                     updated_at = NOW() WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(repository_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("unstar bump: {e}"))?;
            }
            let count: i64 = sqlx::query_scalar("SELECT star_count FROM repositories WHERE id = ?")
                .bind(repository_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| format!("unstar count: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("unstar commit: {e}"))?;
            Ok(count)
        }
        DbPool::Sqlite(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("unstar begin: {e}"))?;
            let deleted = sqlx::query(
                "DELETE FROM repository_stars WHERE user_id = ?1 AND repository_id = ?2",
            )
            .bind(user_id)
            .bind(repository_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("unstar delete: {e}"))?
            .rows_affected();
            if deleted > 0 {
                sqlx::query(
                    "UPDATE repositories SET star_count = MAX(star_count - 1, 0),
                     updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
                     WHERE id = ?1 AND deleted_at IS NULL",
                )
                .bind(repository_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("unstar bump: {e}"))?;
            }
            let count: i64 = sqlx::query_scalar(
                "SELECT star_count FROM repositories WHERE id = ?1",
            )
            .bind(repository_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("unstar count: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("unstar commit: {e}"))?;
            Ok(count)
        }
    }
}

pub async fn get_star_count(pool: &DbPool, repository_id: &str) -> Result<i64, String> {
    match pool {
        DbPool::Postgres(p) => sqlx::query_scalar(
            "SELECT COALESCE(star_count, 0) FROM repositories WHERE id = $1",
        )
        .bind(repository_id)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("get star_count: {e}"))
        .map(|o| o.unwrap_or(0)),
        DbPool::MySql(p) => sqlx::query_scalar(
            "SELECT COALESCE(star_count, 0) FROM repositories WHERE id = ?",
        )
        .bind(repository_id)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("get star_count: {e}"))
        .map(|o| o.unwrap_or(0)),
        DbPool::Sqlite(p) => sqlx::query_scalar(
            "SELECT COALESCE(star_count, 0) FROM repositories WHERE id = ?1",
        )
        .bind(repository_id)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("get star_count: {e}"))
        .map(|o| o.unwrap_or(0)),
    }
}

pub async fn has_starred(
    pool: &DbPool,
    user_id: &str,
    repository_id: &str,
) -> Result<bool, String> {
    let count: i64 = match pool {
        DbPool::Postgres(p) => sqlx::query_scalar(
            "SELECT COUNT(*) FROM repository_stars WHERE user_id = $1 AND repository_id = $2",
        )
        .bind(user_id)
        .bind(repository_id)
        .fetch_one(p)
        .await
        .map_err(|e| format!("has_starred: {e}"))?,
        DbPool::MySql(p) => sqlx::query_scalar(
            "SELECT COUNT(*) FROM repository_stars WHERE user_id = ? AND repository_id = ?",
        )
        .bind(user_id)
        .bind(repository_id)
        .fetch_one(p)
        .await
        .map_err(|e| format!("has_starred: {e}"))?,
        DbPool::Sqlite(p) => sqlx::query_scalar(
            "SELECT COUNT(*) FROM repository_stars WHERE user_id = ?1 AND repository_id = ?2",
        )
        .bind(user_id)
        .bind(repository_id)
        .fetch_one(p)
        .await
        .map_err(|e| format!("has_starred: {e}"))?,
    };
    Ok(count > 0)
}

pub async fn get_fork_network_id(
    pool: &DbPool,
    repository_id: &str,
) -> Result<Option<String>, String> {
    match pool {
        DbPool::Postgres(p) => sqlx::query_scalar(
            "SELECT fork_network_id FROM repositories WHERE id = $1",
        )
        .bind(repository_id)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("get fork_network_id: {e}")),
        DbPool::MySql(p) => {
            sqlx::query_scalar("SELECT fork_network_id FROM repositories WHERE id = ?")
                .bind(repository_id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("get fork_network_id: {e}"))
        }
        DbPool::Sqlite(p) => sqlx::query_scalar(
            "SELECT fork_network_id FROM repositories WHERE id = ?1",
        )
        .bind(repository_id)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("get fork_network_id: {e}")),
    }
}

/// Set fork_network_id (roots: own id; forks: source network root).
pub async fn set_fork_network_id(
    pool: &DbPool,
    repository_id: &str,
    network_id: &str,
) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query("UPDATE repositories SET fork_network_id = $2 WHERE id = $1")
                .bind(repository_id)
                .bind(network_id)
                .execute(p)
                .await
                .map_err(|e| format!("set fork_network_id: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query("UPDATE repositories SET fork_network_id = ? WHERE id = ?")
                .bind(network_id)
                .bind(repository_id)
                .execute(p)
                .await
                .map_err(|e| format!("set fork_network_id: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query("UPDATE repositories SET fork_network_id = ?2 WHERE id = ?1")
                .bind(repository_id)
                .bind(network_id)
                .execute(p)
                .await
                .map_err(|e| format!("set fork_network_id: {e}"))?;
        }
    }
    Ok(())
}

/// Active fork for (owner_id, fork_network_id), if any.
pub async fn find_active_fork_in_network(
    pool: &DbPool,
    owner_id: &str,
    fork_network_id: &str,
) -> Result<Option<RepositoryRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT id FROM repositories
                 WHERE owner_id = $1 AND fork_network_id = $2
                   AND forked_from_repo_id IS NOT NULL AND deleted_at IS NULL
                 LIMIT 1",
            )
            .bind(owner_id)
            .bind(fork_network_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find fork in network: {e}"))?;
            match row {
                Some(r) => {
                    let id: String = r.try_get("id").map_err(|e| format!("row: {e}"))?;
                    repositories::find_by_id(pool, &id).await
                }
                None => Ok(None),
            }
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                "SELECT id FROM repositories
                 WHERE owner_id = ? AND fork_network_id = ?
                   AND forked_from_repo_id IS NOT NULL AND deleted_at IS NULL
                 LIMIT 1",
            )
            .bind(owner_id)
            .bind(fork_network_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find fork in network: {e}"))?;
            match row {
                Some(r) => {
                    let id: String = r.try_get("id").map_err(|e| format!("row: {e}"))?;
                    repositories::find_by_id(pool, &id).await
                }
                None => Ok(None),
            }
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT id FROM repositories
                 WHERE owner_id = ?1 AND fork_network_id = ?2
                   AND forked_from_repo_id IS NOT NULL AND deleted_at IS NULL
                 LIMIT 1",
            )
            .bind(owner_id)
            .bind(fork_network_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find fork in network: {e}"))?;
            match row {
                Some(r) => {
                    let id: String = r.try_get("id").map_err(|e| format!("row: {e}"))?;
                    repositories::find_by_id(pool, &id).await
                }
                None => Ok(None),
            }
        }
    }
}

/// Starred repo ids for a user, newest-starred first, with offset/limit.
pub async fn list_starred_repo_ids(
    pool: &DbPool,
    user_id: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<String>, String> {
    match pool {
        DbPool::Postgres(p) => sqlx::query_scalar(
            "SELECT repository_id FROM repository_stars
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(p)
        .await
        .map_err(|e| format!("list starred: {e}")),
        DbPool::MySql(p) => sqlx::query_scalar(
            "SELECT repository_id FROM repository_stars
             WHERE user_id = ?
             ORDER BY created_at DESC
             LIMIT ? OFFSET ?",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(p)
        .await
        .map_err(|e| format!("list starred: {e}")),
        DbPool::Sqlite(p) => sqlx::query_scalar(
            "SELECT repository_id FROM repository_stars
             WHERE user_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2 OFFSET ?3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(p)
        .await
        .map_err(|e| format!("list starred: {e}")),
    }
}

/// Public explore listing: star_count DESC, updated_at DESC.
pub async fn list_explore(
    pool: &DbPool,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<RepositoryRow>, String> {
    let q_pat = q.map(|s| {
        let escaped = s.replace('%', r"\%").replace('_', r"\_");
        format!("%{escaped}%")
    });
    match pool {
        DbPool::Postgres(p) => {
            let rows = if let Some(ref pat) = q_pat {
                sqlx::query(
                    "SELECT id FROM repositories
                     WHERE deleted_at IS NULL AND lower(visibility) = 'public'
                       AND (name ILIKE $1 ESCAPE '\\' OR COALESCE(description, '') ILIKE $1 ESCAPE '\\')
                     ORDER BY star_count DESC, updated_at DESC
                     LIMIT $2 OFFSET $3",
                )
                .bind(pat)
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
            } else {
                sqlx::query(
                    "SELECT id FROM repositories
                     WHERE deleted_at IS NULL AND lower(visibility) = 'public'
                     ORDER BY star_count DESC, updated_at DESC
                     LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
            }
            .map_err(|e| format!("explore: {e}"))?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                let id: String = r.try_get("id").map_err(|e| format!("row: {e}"))?;
                if let Some(row) = repositories::find_by_id(pool, &id).await? {
                    out.push(row);
                }
            }
            Ok(out)
        }
        DbPool::MySql(p) => {
            let rows = if let Some(ref pat) = q_pat {
                sqlx::query(
                    "SELECT id FROM repositories
                     WHERE deleted_at IS NULL AND LOWER(visibility) = 'public'
                       AND (name LIKE ? OR COALESCE(description, '') LIKE ?)
                     ORDER BY star_count DESC, updated_at DESC
                     LIMIT ? OFFSET ?",
                )
                .bind(pat)
                .bind(pat)
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
            } else {
                sqlx::query(
                    "SELECT id FROM repositories
                     WHERE deleted_at IS NULL AND LOWER(visibility) = 'public'
                     ORDER BY star_count DESC, updated_at DESC
                     LIMIT ? OFFSET ?",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
            }
            .map_err(|e| format!("explore: {e}"))?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                let id: String = r.try_get("id").map_err(|e| format!("row: {e}"))?;
                if let Some(row) = repositories::find_by_id(pool, &id).await? {
                    out.push(row);
                }
            }
            Ok(out)
        }
        DbPool::Sqlite(p) => {
            let rows = if let Some(ref pat) = q_pat {
                sqlx::query(
                    "SELECT id FROM repositories
                     WHERE deleted_at IS NULL AND lower(visibility) = 'public'
                       AND (name LIKE ?1 ESCAPE '\\' OR COALESCE(description, '') LIKE ?1 ESCAPE '\\')
                     ORDER BY star_count DESC, updated_at DESC
                     LIMIT ?2 OFFSET ?3",
                )
                .bind(pat)
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
            } else {
                sqlx::query(
                    "SELECT id FROM repositories
                     WHERE deleted_at IS NULL AND lower(visibility) = 'public'
                     ORDER BY star_count DESC, updated_at DESC
                     LIMIT ?1 OFFSET ?2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
            }
            .map_err(|e| format!("explore: {e}"))?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                let id: String = r.try_get("id").map_err(|e| format!("row: {e}"))?;
                if let Some(row) = repositories::find_by_id(pool, &id).await? {
                    out.push(row);
                }
            }
            Ok(out)
        }
    }
}
