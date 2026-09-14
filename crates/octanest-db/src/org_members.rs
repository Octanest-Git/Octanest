//! Organization membership helpers via `DbPool` match.

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct OrgMemberRow {
    pub org_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: String,
}

macro_rules! map_member {
    ($row:expr) => {{
        let row = $row;
        OrgMemberRow {
            org_id: row.try_get("org_id").map_err(|e| format!("org member row: {e}"))?,
            user_id: row
                .try_get("user_id")
                .map_err(|e| format!("org member row: {e}"))?,
            role: row.try_get("role").map_err(|e| format!("org member row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("org member row: {e}"))?,
        }
    }};
}

const MEMBER_SELECT_PG: &str = "SELECT org_id, user_id, role,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
FROM organization_members";

const MEMBER_SELECT_MYSQL: &str = "SELECT org_id, user_id, role,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at
FROM organization_members";

const MEMBER_SELECT_SQLITE: &str = "SELECT org_id, user_id, role,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at
FROM organization_members";

/// Insert a membership row (`owner` | `admin` | `member` — D-ORG-02a).
pub async fn insert_member(
    pool: &DbPool,
    org_id: &str,
    user_id: &str,
    role: &str,
) -> Result<OrgMemberRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO organization_members (org_id, user_id, role)
VALUES ($1, $2, $3)",
            )
            .bind(org_id)
            .bind(user_id)
            .bind(role)
            .execute(p)
            .await
            .map_err(|e| format!("insert org member failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO organization_members (org_id, user_id, role)
VALUES (?, ?, ?)",
            )
            .bind(org_id)
            .bind(user_id)
            .bind(role)
            .execute(p)
            .await
            .map_err(|e| format!("insert org member failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO organization_members (org_id, user_id, role)
VALUES (?1, ?2, ?3)",
            )
            .bind(org_id)
            .bind(user_id)
            .bind(role)
            .execute(p)
            .await
            .map_err(|e| format!("insert org member failed: {e}"))?;
        }
    }
    find_member(pool, org_id, user_id)
        .await?
        .ok_or_else(|| "insert org member failed: row missing after insert".into())
}

/// Insert the creating user as Owner (ORG-01 / D-ORG-02a).
pub async fn insert_owner_membership(
    pool: &DbPool,
    org_id: &str,
    user_id: &str,
) -> Result<OrgMemberRow, String> {
    insert_member(pool, org_id, user_id, "owner").await
}

/// Membership role for ACL (`owner` | `admin` | `member`), or `None` if not a member.
pub async fn find_member_role(
    pool: &DbPool,
    org_id: &str,
    user_id: &str,
) -> Result<Option<String>, String> {
    Ok(find_member(pool, org_id, user_id)
        .await?
        .map(|m| m.role))
}

pub async fn find_member(
    pool: &DbPool,
    org_id: &str,
    user_id: &str,
) -> Result<Option<OrgMemberRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!(
                "{MEMBER_SELECT_PG} WHERE org_id = $1 AND user_id = $2"
            ))
            .bind(org_id)
            .bind(user_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find org member failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_member!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!(
                "{MEMBER_SELECT_MYSQL} WHERE org_id = ? AND user_id = ?"
            ))
            .bind(org_id)
            .bind(user_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find org member failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_member!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!(
                "{MEMBER_SELECT_SQLITE} WHERE org_id = ?1 AND user_id = ?2"
            ))
            .bind(org_id)
            .bind(user_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find org member failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_member!(&r)),
                None => None,
            })
        }
    }
}
