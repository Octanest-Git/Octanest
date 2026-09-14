//! Dual-scope labels + issue label/assignee assignment helpers (D-ISS-05, D-ISS-06).

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct LabelRow {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub org_id: Option<String>,
    pub repo_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

macro_rules! map_label {
    ($row:expr) => {{
        let row = $row;
        LabelRow {
            id: row.try_get("id").map_err(|e| format!("label row: {e}"))?,
            name: row.try_get("name").map_err(|e| format!("label row: {e}"))?,
            color: row
                .try_get("color")
                .map_err(|e| format!("label row: {e}"))?,
            description: row
                .try_get("description")
                .map_err(|e| format!("label row: {e}"))?,
            org_id: row
                .try_get("org_id")
                .map_err(|e| format!("label row: {e}"))?,
            repo_id: row
                .try_get("repo_id")
                .map_err(|e| format!("label row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("label row: {e}"))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("label row: {e}"))?,
        }
    }};
}

const LABEL_SELECT_PG: &str = "SELECT id, name, color, description, org_id, repo_id,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at,
       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS updated_at
FROM labels";

const LABEL_SELECT_MYSQL: &str = "SELECT id, name, color, description, org_id, repo_id,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
       DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
FROM labels";

const LABEL_SELECT_SQLITE: &str = "SELECT id, name, color, description, org_id, repo_id,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
FROM labels";

/// Insert an org-scoped or repo-local label. Exactly one of `org_id` / `repo_id` must be set.
pub async fn insert_label(
    pool: &DbPool,
    id: &str,
    name: &str,
    color: &str,
    description: &str,
    org_id: Option<&str>,
    repo_id: Option<&str>,
) -> Result<LabelRow, String> {
    match (org_id, repo_id) {
        (Some(_), None) | (None, Some(_)) => {}
        _ => {
            return Err(
                "label scope requires exactly one of org_id or repo_id".into(),
            );
        }
    }
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO labels (id, name, color, description, org_id, repo_id)
VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(id)
            .bind(name)
            .bind(color)
            .bind(description)
            .bind(org_id)
            .bind(repo_id)
            .execute(p)
            .await
            .map_err(|e| format!("insert label failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO labels (id, name, color, description, org_id, repo_id)
VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(name)
            .bind(color)
            .bind(description)
            .bind(org_id)
            .bind(repo_id)
            .execute(p)
            .await
            .map_err(|e| format!("insert label failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO labels (id, name, color, description, org_id, repo_id)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(id)
            .bind(name)
            .bind(color)
            .bind(description)
            .bind(org_id)
            .bind(repo_id)
            .execute(p)
            .await
            .map_err(|e| format!("insert label failed: {e}"))?;
        }
    }
    find_label_by_id(pool, id)
        .await?
        .ok_or_else(|| "insert label failed: row missing after insert".into())
}

pub async fn find_label_by_id(pool: &DbPool, id: &str) -> Result<Option<LabelRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{LABEL_SELECT_PG} WHERE id = $1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find label by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_label!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{LABEL_SELECT_MYSQL} WHERE id = ?"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find label by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_label!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{LABEL_SELECT_SQLITE} WHERE id = ?1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find label by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_label!(&r)),
                None => None,
            })
        }
    }
}

/// Replace the label set on an issue (M:N via `issue_labels`).
pub async fn set_issue_labels(
    pool: &DbPool,
    issue_id: &str,
    label_ids: &[String],
) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin set issue labels tx failed: {e}"))?;
            sqlx::query("DELETE FROM issue_labels WHERE issue_id = $1")
                .bind(issue_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear issue labels failed: {e}"))?;
            for label_id in label_ids {
                sqlx::query(
                    "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)",
                )
                .bind(issue_id)
                .bind(label_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert issue label failed: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("commit set issue labels tx failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin set issue labels tx failed: {e}"))?;
            sqlx::query("DELETE FROM issue_labels WHERE issue_id = ?")
                .bind(issue_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear issue labels failed: {e}"))?;
            for label_id in label_ids {
                sqlx::query(
                    "INSERT INTO issue_labels (issue_id, label_id) VALUES (?, ?)",
                )
                .bind(issue_id)
                .bind(label_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert issue label failed: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("commit set issue labels tx failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin set issue labels tx failed: {e}"))?;
            sqlx::query("DELETE FROM issue_labels WHERE issue_id = ?1")
                .bind(issue_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear issue labels failed: {e}"))?;
            for label_id in label_ids {
                sqlx::query(
                    "INSERT INTO issue_labels (issue_id, label_id) VALUES (?1, ?2)",
                )
                .bind(issue_id)
                .bind(label_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert issue label failed: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("commit set issue labels tx failed: {e}"))?;
        }
    }
    Ok(())
}

/// Replace assignees on an issue (M:N via `issue_assignees`).
pub async fn set_issue_assignees(
    pool: &DbPool,
    issue_id: &str,
    user_ids: &[String],
) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin set issue assignees tx failed: {e}"))?;
            sqlx::query("DELETE FROM issue_assignees WHERE issue_id = $1")
                .bind(issue_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear issue assignees failed: {e}"))?;
            for user_id in user_ids {
                sqlx::query(
                    "INSERT INTO issue_assignees (issue_id, user_id) VALUES ($1, $2)",
                )
                .bind(issue_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert issue assignee failed: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("commit set issue assignees tx failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin set issue assignees tx failed: {e}"))?;
            sqlx::query("DELETE FROM issue_assignees WHERE issue_id = ?")
                .bind(issue_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear issue assignees failed: {e}"))?;
            for user_id in user_ids {
                sqlx::query(
                    "INSERT INTO issue_assignees (issue_id, user_id) VALUES (?, ?)",
                )
                .bind(issue_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert issue assignee failed: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("commit set issue assignees tx failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin set issue assignees tx failed: {e}"))?;
            sqlx::query("DELETE FROM issue_assignees WHERE issue_id = ?1")
                .bind(issue_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear issue assignees failed: {e}"))?;
            for user_id in user_ids {
                sqlx::query(
                    "INSERT INTO issue_assignees (issue_id, user_id) VALUES (?1, ?2)",
                )
                .bind(issue_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert issue assignee failed: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("commit set issue assignees tx failed: {e}"))?;
        }
    }
    Ok(())
}
