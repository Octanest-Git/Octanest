//! Organization CRUD via `DbPool` match — dialect branching stays in this crate.

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct OrganizationRow {
    pub id: String,
    pub slug: String,
    pub display_name: String,
    pub member_base_permission: String,
    pub created_at: String,
    pub updated_at: String,
}

macro_rules! map_org {
    ($row:expr) => {{
        let row = $row;
        OrganizationRow {
            id: row.try_get("id").map_err(|e| format!("org row: {e}"))?,
            slug: row.try_get("slug").map_err(|e| format!("org row: {e}"))?,
            display_name: row
                .try_get("display_name")
                .map_err(|e| format!("org row: {e}"))?,
            member_base_permission: row
                .try_get("member_base_permission")
                .map_err(|e| format!("org row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("org row: {e}"))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("org row: {e}"))?,
        }
    }};
}

const ORG_SELECT_PG: &str = "SELECT id, slug, display_name, member_base_permission,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at,
       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS updated_at
FROM organizations";

const ORG_SELECT_MYSQL: &str = "SELECT id, slug, display_name, member_base_permission,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
       DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
FROM organizations";

const ORG_SELECT_SQLITE: &str = "SELECT id, slug, display_name, member_base_permission,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
FROM organizations";

/// Org-level Member base permission string (`none` | `read` | `write`), if org exists.
pub async fn find_member_base_permission(
    pool: &DbPool,
    org_id: &str,
) -> Result<Option<String>, String> {
    Ok(find_by_id(pool, org_id)
        .await?
        .map(|o| o.member_base_permission))
}

/// Insert an organization. Default `member_base_permission` is `none` (D-ORG-02b).
pub async fn insert_organization(
    pool: &DbPool,
    id: &str,
    slug: &str,
    display_name: &str,
    member_base_permission: &str,
) -> Result<OrganizationRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO organizations (id, slug, display_name, member_base_permission)
VALUES ($1, $2, $3, $4)",
            )
            .bind(id)
            .bind(slug)
            .bind(display_name)
            .bind(member_base_permission)
            .execute(p)
            .await
            .map_err(|e| format!("insert organization failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO organizations (id, slug, display_name, member_base_permission)
VALUES (?, ?, ?, ?)",
            )
            .bind(id)
            .bind(slug)
            .bind(display_name)
            .bind(member_base_permission)
            .execute(p)
            .await
            .map_err(|e| format!("insert organization failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO organizations (id, slug, display_name, member_base_permission)
VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(id)
            .bind(slug)
            .bind(display_name)
            .bind(member_base_permission)
            .execute(p)
            .await
            .map_err(|e| format!("insert organization failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "insert organization failed: row missing after insert".into())
}

pub async fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<OrganizationRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{ORG_SELECT_PG} WHERE id = $1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find organization failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_org!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{ORG_SELECT_MYSQL} WHERE id = ?"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find organization failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_org!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{ORG_SELECT_SQLITE} WHERE id = ?1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find organization failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_org!(&r)),
                None => None,
            })
        }
    }
}

/// Case-insensitive slug lookup (D-ORG-01 shared namespace).
pub async fn find_by_slug(pool: &DbPool, slug: &str) -> Result<Option<OrganizationRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{ORG_SELECT_PG} WHERE lower(slug) = lower($1)"))
                .bind(slug)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find organization by slug failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_org!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{ORG_SELECT_MYSQL} WHERE LOWER(slug) = LOWER(?)"))
                .bind(slug)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find organization by slug failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_org!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{ORG_SELECT_SQLITE} WHERE lower(slug) = lower(?1)"))
                .bind(slug)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find organization by slug failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_org!(&r)),
                None => None,
            })
        }
    }
}
