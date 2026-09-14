//! Per-repository collaborator grants (`read` | `write` | `admin` — D-ORG-02c).

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct RepoCollaboratorRow {
    pub repo_id: String,
    pub user_id: String,
    pub permission: String,
    pub created_at: String,
}

macro_rules! map_collaborator {
    ($row:expr) => {{
        let row = $row;
        RepoCollaboratorRow {
            repo_id: row
                .try_get("repo_id")
                .map_err(|e| format!("repo collaborator row: {e}"))?,
            user_id: row
                .try_get("user_id")
                .map_err(|e| format!("repo collaborator row: {e}"))?,
            permission: row
                .try_get("permission")
                .map_err(|e| format!("repo collaborator row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("repo collaborator row: {e}"))?,
        }
    }};
}

const COLLAB_SELECT_PG: &str = "SELECT repo_id, user_id, permission,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
FROM repository_collaborators";

const COLLAB_SELECT_MYSQL: &str = "SELECT repo_id, user_id, permission,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at
FROM repository_collaborators";

const COLLAB_SELECT_SQLITE: &str = "SELECT repo_id, user_id, permission,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at
FROM repository_collaborators";

/// Look up a collaborator grant for `(repo_id, user_id)`.
pub async fn find_collaborator(
    pool: &DbPool,
    repo_id: &str,
    user_id: &str,
) -> Result<Option<RepoCollaboratorRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!(
                "{COLLAB_SELECT_PG} WHERE repo_id = $1 AND user_id = $2"
            ))
            .bind(repo_id)
            .bind(user_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repo collaborator failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_collaborator!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!(
                "{COLLAB_SELECT_MYSQL} WHERE repo_id = ? AND user_id = ?"
            ))
            .bind(repo_id)
            .bind(user_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repo collaborator failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_collaborator!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!(
                "{COLLAB_SELECT_SQLITE} WHERE repo_id = ?1 AND user_id = ?2"
            ))
            .bind(repo_id)
            .bind(user_id)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find repo collaborator failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_collaborator!(&r)),
                None => None,
            })
        }
    }
}
