//! Repository CRUD via `DbPool` match — dialect branching stays in this crate.

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct RepositoryRow {
    pub id: String,
    pub owner_id: String,
    /// Polymorphic owner discriminant: `user` | `org` (D-ORG-01).
    pub owner_type: String,
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
            owner_type: row
                .try_get("owner_type")
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

const REPO_SELECT_PG: &str = "SELECT id, owner_id, owner_type, name, visibility, description, default_branch,
       CASE WHEN deleted_at IS NULL THEN NULL
            ELSE to_char(deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END AS deleted_at,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at,
       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS updated_at
FROM repositories";

const REPO_SELECT_MYSQL: &str = "SELECT id, owner_id, owner_type, name, visibility, description, default_branch,
       CASE WHEN deleted_at IS NULL THEN NULL
            ELSE DATE_FORMAT(deleted_at, '%Y-%m-%dT%H:%i:%sZ') END AS deleted_at,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
       DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
FROM repositories";

const REPO_SELECT_SQLITE: &str = "SELECT id, owner_id, owner_type, name, visibility, description, default_branch,
       CASE WHEN deleted_at IS NULL THEN NULL
            ELSE strftime('%Y-%m-%dT%H:%M:%SZ', deleted_at) END AS deleted_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
FROM repositories";

pub async fn insert_repository(
    pool: &DbPool,
    id: &str,
    owner_id: &str,
    owner_type: &str,
    name: &str,
    visibility: &str,
    description: &str,
    default_branch: &str,
) -> Result<RepositoryRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO repositories (id, owner_id, owner_type, name, visibility, description, default_branch)
VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(id)
            .bind(owner_id)
            .bind(owner_type)
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
                "INSERT INTO repositories (id, owner_id, owner_type, name, visibility, description, default_branch)
VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(owner_id)
            .bind(owner_type)
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
                "INSERT INTO repositories (id, owner_id, owner_type, name, visibility, description, default_branch)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .bind(id)
            .bind(owner_id)
            .bind(owner_type)
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

/// Update repository name for a non-deleted row (GIT-16 / D-REL-07).
pub async fn update_name(pool: &DbPool, id: &str, name: &str) -> Result<RepositoryRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET name = $2, updated_at = now()
WHERE id = $1 AND deleted_at IS NULL",
            )
            .bind(id)
            .bind(name)
            .execute(p)
            .await
            .map_err(|e| format!("update repository name failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
        DbPool::MySql(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET name = ?, updated_at = NOW()
WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(name)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("update repository name failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
        DbPool::Sqlite(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET name = ?2, updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1 AND deleted_at IS NULL",
            )
            .bind(id)
            .bind(name)
            .execute(p)
            .await
            .map_err(|e| format!("update repository name failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "repository not found after name update".into())
}

/// Update visibility for a non-deleted repository (D-26).
pub async fn update_visibility(
    pool: &DbPool,
    id: &str,
    visibility: &str,
) -> Result<RepositoryRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET visibility = $2, updated_at = now()
WHERE id = $1 AND deleted_at IS NULL",
            )
            .bind(id)
            .bind(visibility)
            .execute(p)
            .await
            .map_err(|e| format!("update repository visibility failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
        DbPool::MySql(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET visibility = ?, updated_at = NOW()
WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(visibility)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("update repository visibility failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
        DbPool::Sqlite(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET visibility = ?2, updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1 AND deleted_at IS NULL",
            )
            .bind(id)
            .bind(visibility)
            .execute(p)
            .await
            .map_err(|e| format!("update repository visibility failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "repository not found after visibility update".into())
}

/// Soft-delete: set `deleted_at` (disk purge deferred — D-35).
pub async fn soft_delete(pool: &DbPool, id: &str) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET deleted_at = now(), updated_at = now()
WHERE id = $1 AND deleted_at IS NULL",
            )
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("soft-delete repository failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
        DbPool::MySql(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET deleted_at = NOW(), updated_at = NOW()
WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("soft-delete repository failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
        DbPool::Sqlite(p) => {
            let n = sqlx::query(
                "UPDATE repositories SET deleted_at = strftime('%Y-%m-%d %H:%M:%S','now'),
updated_at = strftime('%Y-%m-%d %H:%M:%S','now')
WHERE id = ?1 AND deleted_at IS NULL",
            )
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("soft-delete repository failed: {e}"))?
            .rows_affected();
            if n == 0 {
                return Err("repository not found".into());
            }
        }
    }
    Ok(())
}

/// On-disk identity for a repository row (active or soft-deleted).
#[derive(Debug, Clone)]
pub struct RepoDiskRef {
    pub id: String,
    pub owner_username: String,
    pub name: String,
    pub deleted_at: Option<String>,
}

/// Resolve disk-path slug for user- or org-owned repos (D-ORG-01).
const DISK_REF_SELECT_PG: &str = "SELECT r.id,
       CASE WHEN r.owner_type = 'org' THEN o.slug ELSE u.username END AS owner_username,
       r.name,
       CASE WHEN r.deleted_at IS NULL THEN NULL
            ELSE to_char(r.deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END AS deleted_at
FROM repositories r
LEFT JOIN users u ON r.owner_type = 'user' AND u.id = r.owner_id
LEFT JOIN organizations o ON r.owner_type = 'org' AND o.id = r.owner_id";

const DISK_REF_SELECT_MYSQL: &str = "SELECT r.id,
       CASE WHEN r.owner_type = 'org' THEN o.slug ELSE u.username END AS owner_username,
       r.name,
       CASE WHEN r.deleted_at IS NULL THEN NULL
            ELSE DATE_FORMAT(r.deleted_at, '%Y-%m-%dT%H:%i:%sZ') END AS deleted_at
FROM repositories r
LEFT JOIN users u ON r.owner_type = 'user' AND u.id = r.owner_id
LEFT JOIN organizations o ON r.owner_type = 'org' AND o.id = r.owner_id";

const DISK_REF_SELECT_SQLITE: &str = "SELECT r.id,
       CASE WHEN r.owner_type = 'org' THEN o.slug ELSE u.username END AS owner_username,
       r.name,
       CASE WHEN r.deleted_at IS NULL THEN NULL
            ELSE strftime('%Y-%m-%dT%H:%M:%SZ', r.deleted_at) END AS deleted_at
FROM repositories r
LEFT JOIN users u ON r.owner_type = 'user' AND u.id = r.owner_id
LEFT JOIN organizations o ON r.owner_type = 'org' AND o.id = r.owner_id";

/// All repository rows that still own a disk path (active + soft-deleted).
pub async fn list_repo_disk_refs(pool: &DbPool) -> Result<Vec<RepoDiskRef>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let rows = sqlx::query(DISK_REF_SELECT_PG)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list repo disk refs failed: {e}"))?;
            rows.into_iter()
                .map(|row| {
                    Ok(RepoDiskRef {
                        id: row.try_get("id").map_err(|e| format!("repo disk ref: {e}"))?,
                        owner_username: row
                            .try_get("owner_username")
                            .map_err(|e| format!("repo disk ref: {e}"))?,
                        name: row.try_get("name").map_err(|e| format!("repo disk ref: {e}"))?,
                        deleted_at: row
                            .try_get("deleted_at")
                            .map_err(|e| format!("repo disk ref: {e}"))?,
                    })
                })
                .collect()
        }
        DbPool::MySql(p) => {
            let rows = sqlx::query(DISK_REF_SELECT_MYSQL)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list repo disk refs failed: {e}"))?;
            rows.into_iter()
                .map(|row| {
                    Ok(RepoDiskRef {
                        id: row.try_get("id").map_err(|e| format!("repo disk ref: {e}"))?,
                        owner_username: row
                            .try_get("owner_username")
                            .map_err(|e| format!("repo disk ref: {e}"))?,
                        name: row.try_get("name").map_err(|e| format!("repo disk ref: {e}"))?,
                        deleted_at: row
                            .try_get("deleted_at")
                            .map_err(|e| format!("repo disk ref: {e}"))?,
                    })
                })
                .collect()
        }
        DbPool::Sqlite(p) => {
            let rows = sqlx::query(DISK_REF_SELECT_SQLITE)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list repo disk refs failed: {e}"))?;
            rows.into_iter()
                .map(|row| {
                    Ok(RepoDiskRef {
                        id: row.try_get("id").map_err(|e| format!("repo disk ref: {e}"))?,
                        owner_username: row
                            .try_get("owner_username")
                            .map_err(|e| format!("repo disk ref: {e}"))?,
                        name: row.try_get("name").map_err(|e| format!("repo disk ref: {e}"))?,
                        deleted_at: row
                            .try_get("deleted_at")
                            .map_err(|e| format!("repo disk ref: {e}"))?,
                    })
                })
                .collect()
        }
    }
}

/// Hard-delete a repository row (after disk purge — D-35/D-36).
pub async fn hard_delete(pool: &DbPool, id: &str) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query("DELETE FROM repositories WHERE id = $1")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("hard-delete repository failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query("DELETE FROM repositories WHERE id = ?")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("hard-delete repository failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query("DELETE FROM repositories WHERE id = ?1")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("hard-delete repository failed: {e}"))?;
        }
    }
    Ok(())
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
