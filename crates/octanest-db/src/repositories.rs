//! Repository CRUD via `DbPool` match — dialect branching stays in this crate.

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct RepositoryRow {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub visibility: String,
    pub description: String,
    pub default_branch: String,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

macro_rules! map_repo {
    ($row:expr) => {{
        let row = $row;
        RepositoryRow {
            id: row.try_get("id").map_err(|e| format!("repo row: {e}"))?,
            owner_id: row
                .try_get("owner_id")
                .map_err(|e| format!("repo row: {e}"))?,
            name: row.try_get("name").map_err(|e| format!("repo row: {e}"))?,
            visibility: row
                .try_get("visibility")
                .map_err(|e| format!("repo row: {e}"))?,
            description: row
                .try_get("description")
                .map_err(|e| format!("repo row: {e}"))?,
            default_branch: row
                .try_get("default_branch")
                .map_err(|e| format!("repo row: {e}"))?,
            deleted_at: row
                .try_get("deleted_at")
                .map_err(|e| format!("repo row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("repo row: {e}"))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("repo row: {e}"))?,
        }
    }};
}

const REPO_SELECT_PG: &str = "SELECT id, owner_id, name, visibility, description, default_branch,
       CASE WHEN deleted_at IS NULL THEN NULL
            ELSE to_char(deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END AS deleted_at,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at,
       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS updated_at
FROM repositories";

const REPO_SELECT_MYSQL: &str = "SELECT id, owner_id, name, visibility, description, default_branch,
       CASE WHEN deleted_at IS NULL THEN NULL
            ELSE DATE_FORMAT(deleted_at, '%Y-%m-%dT%H:%i:%sZ') END AS deleted_at,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
       DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
FROM repositories";

const REPO_SELECT_SQLITE: &str = "SELECT id, owner_id, name, visibility, description, default_branch,
       CASE WHEN deleted_at IS NULL THEN NULL
            ELSE strftime('%Y-%m-%dT%H:%M:%SZ', deleted_at) END AS deleted_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
FROM repositories";

pub async fn insert_repository(
    pool: &DbPool,
    id: &str,
    owner_id: &str,
    name: &str,
    visibility: &str,
    description: &str,
    default_branch: &str,
) -> Result<RepositoryRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO repositories (id, owner_id, name, visibility, description, default_branch)
VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(id)
            .bind(owner_id)
            .bind(name)
            .bind(visibility)
            .bind(description)
            .bind(default_branch)
            .execute(p)
            .await
            .map_err(|e| format!("insert repository failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO repositories (id, owner_id, name, visibility, description, default_branch)
VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(owner_id)
            .bind(name)
            .bind(visibility)
            .bind(description)
            .bind(default_branch)
            .execute(p)
            .await
            .map_err(|e| format!("insert repository failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO repositories (id, owner_id, name, visibility, description, default_branch)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(id)
            .bind(owner_id)
            .bind(name)
            .bind(visibility)
            .bind(description)
            .bind(default_branch)
            .execute(p)
            .await
            .map_err(|e| format!("insert repository failed: {e}"))?;
        }
    }
    find_by_owner_and_name(pool, owner_id, name)
        .await?
        .ok_or_else(|| "insert repository failed: row missing after insert".into())
}

/// Lookup by owner + name among non-deleted rows (case-insensitive name match).
pub async fn find_by_owner_and_name(
    pool: &DbPool,
    owner_id: &str,
    name: &str,
) -> Result<Option<RepositoryRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!(
                "{REPO_SELECT_PG} WHERE owner_id = $1 AND lower(name) = lower($2) AND deleted_at IS NULL"
            ))
            .bind(owner_id)
            .bind(name)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repository failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_repo!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!(
                "{REPO_SELECT_MYSQL} WHERE owner_id = ? AND LOWER(name) = LOWER(?) AND deleted_at IS NULL"
            ))
            .bind(owner_id)
            .bind(name)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repository failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_repo!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!(
                "{REPO_SELECT_SQLITE} WHERE owner_id = ?1 AND lower(name) = lower(?2) AND deleted_at IS NULL"
            ))
            .bind(owner_id)
            .bind(name)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repository failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_repo!(&r)),
                None => None,
            })
        }
    }
}

/// List non-deleted repos for an owner, most recently updated first (GIT-01 / D-13).
pub async fn list_by_owner(
    pool: &DbPool,
    owner_id: &str,
) -> Result<Vec<RepositoryRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let rows = sqlx::query(&format!(
                "{REPO_SELECT_PG} WHERE owner_id = $1 AND deleted_at IS NULL ORDER BY updated_at DESC"
            ))
            .bind(owner_id)
            .fetch_all(p)
            .await
            .map_err(|e| format!("list repositories failed: {e}"))?;
            rows.iter().map(|r| Ok(map_repo!(r))).collect()
        }
        DbPool::MySql(p) => {
            let rows = sqlx::query(&format!(
                "{REPO_SELECT_MYSQL} WHERE owner_id = ? AND deleted_at IS NULL ORDER BY updated_at DESC"
            ))
            .bind(owner_id)
            .fetch_all(p)
            .await
            .map_err(|e| format!("list repositories failed: {e}"))?;
            rows.iter().map(|r| Ok(map_repo!(r))).collect()
        }
        DbPool::Sqlite(p) => {
            let rows = sqlx::query(&format!(
                "{REPO_SELECT_SQLITE} WHERE owner_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC"
            ))
            .bind(owner_id)
            .fetch_all(p)
            .await
            .map_err(|e| format!("list repositories failed: {e}"))?;
            rows.iter().map(|r| Ok(map_repo!(r))).collect()
        }
    }
}

pub async fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<RepositoryRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!(
                "{REPO_SELECT_PG} WHERE id = $1 AND deleted_at IS NULL"
            ))
            .bind(id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repository by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_repo!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!(
                "{REPO_SELECT_MYSQL} WHERE id = ? AND deleted_at IS NULL"
            ))
            .bind(id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repository by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_repo!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!(
                "{REPO_SELECT_SQLITE} WHERE id = ?1 AND deleted_at IS NULL"
            ))
            .bind(id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repository by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_repo!(&r)),
                None => None,
            })
        }
    }
}
