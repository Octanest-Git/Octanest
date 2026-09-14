//! Issues + per-repo `#N` counters (D-ISS-01). Dialect SQL only.

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct IssueRow {
    pub id: String,
    pub repo_id: String,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub author_id: String,
    pub closed_at: Option<String>,
    pub closed_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

macro_rules! map_issue {
    ($row:expr) => {{
        let row = $row;
        IssueRow {
            id: row.try_get("id").map_err(|e| format!("issue row: {e}"))?,
            repo_id: row
                .try_get("repo_id")
                .map_err(|e| format!("issue row: {e}"))?,
            number: row
                .try_get::<i64, _>("number")
                .map_err(|e| format!("issue row: {e}"))?,
            title: row
                .try_get("title")
                .map_err(|e| format!("issue row: {e}"))?,
            body: row.try_get("body").map_err(|e| format!("issue row: {e}"))?,
            state: row
                .try_get("state")
                .map_err(|e| format!("issue row: {e}"))?,
            author_id: row
                .try_get("author_id")
                .map_err(|e| format!("issue row: {e}"))?,
            closed_at: row
                .try_get("closed_at")
                .map_err(|e| format!("issue row: {e}"))?,
            closed_by: row
                .try_get("closed_by")
                .map_err(|e| format!("issue row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("issue row: {e}"))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("issue row: {e}"))?,
        }
    }};
}

const ISSUE_SELECT_PG: &str = "SELECT id, repo_id, number, title, body, state, author_id,
       CASE WHEN closed_at IS NULL THEN NULL
            ELSE to_char(closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END AS closed_at,
       closed_by,
       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at,
       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS updated_at
FROM issues";

const ISSUE_SELECT_MYSQL: &str = "SELECT id, repo_id, number, title, body, state, author_id,
       CASE WHEN closed_at IS NULL THEN NULL
            ELSE DATE_FORMAT(closed_at, '%Y-%m-%dT%H:%i:%sZ') END AS closed_at,
       closed_by,
       DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
       DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
FROM issues";

const ISSUE_SELECT_SQLITE: &str = "SELECT id, repo_id, number, title, body, state, author_id,
       CASE WHEN closed_at IS NULL THEN NULL
            ELSE strftime('%Y-%m-%dT%H:%M:%SZ', closed_at) END AS closed_at,
       closed_by,
       strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
       strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
FROM issues";

/// Allocate the next per-repo issue number (monotonic; never reclaims).
pub async fn allocate_next_number(pool: &DbPool, repo_id: &str) -> Result<i64, String> {
    match pool {
        DbPool::Postgres(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin allocate issue number tx failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issue_counters (repo_id, max_number) VALUES ($1, 0)
                 ON CONFLICT (repo_id) DO NOTHING",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("ensure issue_counters failed: {e}"))?;
            let number: i64 = sqlx::query_scalar(
                "UPDATE issue_counters SET max_number = max_number + 1
                 WHERE repo_id = $1 RETURNING max_number",
            )
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("allocate issue number failed: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("commit allocate issue number tx failed: {e}"))?;
            Ok(number)
        }
        DbPool::MySql(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin allocate issue number tx failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issue_counters (repo_id, max_number) VALUES (?, 0)
                 ON DUPLICATE KEY UPDATE repo_id = repo_id",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("ensure issue_counters failed: {e}"))?;
            sqlx::query(
                "UPDATE issue_counters SET max_number = max_number + 1 WHERE repo_id = ?",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("allocate issue number failed: {e}"))?;
            let number: i64 = sqlx::query_scalar(
                "SELECT max_number FROM issue_counters WHERE repo_id = ?",
            )
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("read allocated issue number failed: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("commit allocate issue number tx failed: {e}"))?;
            Ok(number)
        }
        DbPool::Sqlite(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin allocate issue number tx failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issue_counters (repo_id, max_number) VALUES (?1, 0)
                 ON CONFLICT (repo_id) DO NOTHING",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("ensure issue_counters failed: {e}"))?;
            let number: i64 = sqlx::query_scalar(
                "UPDATE issue_counters SET max_number = max_number + 1
                 WHERE repo_id = ?1 RETURNING max_number",
            )
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("allocate issue number failed: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("commit allocate issue number tx failed: {e}"))?;
            Ok(number)
        }
    }
}

/// Insert an issue, allocating `#N` in the same transaction as the counter upsert.
pub async fn insert_issue(
    pool: &DbPool,
    id: &str,
    repo_id: &str,
    author_id: &str,
    title: &str,
    body: &str,
) -> Result<IssueRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin insert issue tx failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issue_counters (repo_id, max_number) VALUES ($1, 0)
                 ON CONFLICT (repo_id) DO NOTHING",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("ensure issue_counters failed: {e}"))?;
            let number: i64 = sqlx::query_scalar(
                "UPDATE issue_counters SET max_number = max_number + 1
                 WHERE repo_id = $1 RETURNING max_number",
            )
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("allocate issue number failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issues (id, repo_id, number, title, body, state, author_id)
VALUES ($1, $2, $3, $4, $5, 'open', $6)",
            )
            .bind(id)
            .bind(repo_id)
            .bind(number)
            .bind(title)
            .bind(body)
            .bind(author_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("insert issue failed: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("commit insert issue tx failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin insert issue tx failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issue_counters (repo_id, max_number) VALUES (?, 0)
                 ON DUPLICATE KEY UPDATE repo_id = repo_id",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("ensure issue_counters failed: {e}"))?;
            sqlx::query(
                "UPDATE issue_counters SET max_number = max_number + 1 WHERE repo_id = ?",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("allocate issue number failed: {e}"))?;
            let number: i64 = sqlx::query_scalar(
                "SELECT max_number FROM issue_counters WHERE repo_id = ?",
            )
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("read allocated issue number failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issues (id, repo_id, number, title, body, state, author_id)
VALUES (?, ?, ?, ?, ?, 'open', ?)",
            )
            .bind(id)
            .bind(repo_id)
            .bind(number)
            .bind(title)
            .bind(body)
            .bind(author_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("insert issue failed: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("commit insert issue tx failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("begin insert issue tx failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issue_counters (repo_id, max_number) VALUES (?1, 0)
                 ON CONFLICT (repo_id) DO NOTHING",
            )
            .bind(repo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("ensure issue_counters failed: {e}"))?;
            let number: i64 = sqlx::query_scalar(
                "UPDATE issue_counters SET max_number = max_number + 1
                 WHERE repo_id = ?1 RETURNING max_number",
            )
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("allocate issue number failed: {e}"))?;
            sqlx::query(
                "INSERT INTO issues (id, repo_id, number, title, body, state, author_id)
VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6)",
            )
            .bind(id)
            .bind(repo_id)
            .bind(number)
            .bind(title)
            .bind(body)
            .bind(author_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("insert issue failed: {e}"))?;
            tx.commit()
                .await
                .map_err(|e| format!("commit insert issue tx failed: {e}"))?;
        }
    }
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| "insert issue failed: row missing after insert".into())
}

pub async fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<IssueRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!("{ISSUE_SELECT_PG} WHERE id = $1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find issue by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_issue!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!("{ISSUE_SELECT_MYSQL} WHERE id = ?"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find issue by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_issue!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!("{ISSUE_SELECT_SQLITE} WHERE id = ?1"))
                .bind(id)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find issue by id failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_issue!(&r)),
                None => None,
            })
        }
    }
}

/// Look up an issue by repo id + per-repo number.
pub async fn find_by_repo_number(
    pool: &DbPool,
    repo_id: &str,
    number: i64,
) -> Result<Option<IssueRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&format!(
                "{ISSUE_SELECT_PG} WHERE repo_id = $1 AND number = $2"
            ))
            .bind(repo_id)
            .bind(number)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find issue by number failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_issue!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!(
                "{ISSUE_SELECT_MYSQL} WHERE repo_id = ? AND number = ?"
            ))
            .bind(repo_id)
            .bind(number)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find issue by number failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_issue!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!(
                "{ISSUE_SELECT_SQLITE} WHERE repo_id = ?1 AND number = ?2"
            ))
            .bind(repo_id)
            .bind(number)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find issue by number failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_issue!(&r)),
                None => None,
            })
        }
    }
}

/// List issues for a repo. `state_filter`: `open` | `closed` | `all` (default open).
/// Sorted newest-updated first (D-ISS-18). Returns `(rows, total)`.
pub async fn list_for_repo(
    pool: &DbPool,
    repo_id: &str,
    state_filter: &str,
    offset: i64,
    limit: i64,
) -> Result<(Vec<IssueRow>, i64), String> {
    let offset = offset.max(0);
    let limit = if limit <= 0 { 30 } else { limit.min(100) };
    let state = match state_filter.trim() {
        "" | "open" => Some("open"),
        "closed" => Some("closed"),
        "all" => None,
        other => return Err(format!("invalid issue state filter: {other}")),
    };

    match pool {
        DbPool::Postgres(p) => {
            let total: i64 = match state {
                Some(s) => sqlx::query_scalar(
                    "SELECT COUNT(*)::bigint FROM issues WHERE repo_id = $1 AND state = $2",
                )
                .bind(repo_id)
                .bind(s)
                .fetch_one(p)
                .await
                .map_err(|e| format!("count issues failed: {e}"))?,
                None => sqlx::query_scalar(
                    "SELECT COUNT(*)::bigint FROM issues WHERE repo_id = $1",
                )
                .bind(repo_id)
                .fetch_one(p)
                .await
                .map_err(|e| format!("count issues failed: {e}"))?,
            };
            let sql = match state {
                Some(_) => format!(
                    "{ISSUE_SELECT_PG} WHERE repo_id = $1 AND state = $2
 ORDER BY updated_at DESC OFFSET $3 LIMIT $4"
                ),
                None => format!(
                    "{ISSUE_SELECT_PG} WHERE repo_id = $1
 ORDER BY updated_at DESC OFFSET $2 LIMIT $3"
                ),
            };
            let rows = match state {
                Some(s) => {
                    sqlx::query(&sql)
                        .bind(repo_id)
                        .bind(s)
                        .bind(offset)
                        .bind(limit)
                        .fetch_all(p)
                        .await
                }
                None => {
                    sqlx::query(&sql)
                        .bind(repo_id)
                        .bind(offset)
                        .bind(limit)
                        .fetch_all(p)
                        .await
                }
            }
            .map_err(|e| format!("list issues failed: {e}"))?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(map_issue!(&r));
            }
            Ok((out, total))
        }
        DbPool::MySql(p) => {
            let total: i64 = match state {
                Some(s) => sqlx::query_scalar(
                    "SELECT COUNT(*) FROM issues WHERE repo_id = ? AND state = ?",
                )
                .bind(repo_id)
                .bind(s)
                .fetch_one(p)
                .await
                .map_err(|e| format!("count issues failed: {e}"))?,
                None => sqlx::query_scalar("SELECT COUNT(*) FROM issues WHERE repo_id = ?")
                    .bind(repo_id)
                    .fetch_one(p)
                    .await
                    .map_err(|e| format!("count issues failed: {e}"))?,
            };
            let sql = match state {
                Some(_) => format!(
                    "{ISSUE_SELECT_MYSQL} WHERE repo_id = ? AND state = ?
 ORDER BY updated_at DESC LIMIT ? OFFSET ?"
                ),
                None => format!(
                    "{ISSUE_SELECT_MYSQL} WHERE repo_id = ?
 ORDER BY updated_at DESC LIMIT ? OFFSET ?"
                ),
            };
            let rows = match state {
                Some(s) => {
                    sqlx::query(&sql)
                        .bind(repo_id)
                        .bind(s)
                        .bind(limit)
                        .bind(offset)
                        .fetch_all(p)
                        .await
                }
                None => {
                    sqlx::query(&sql)
                        .bind(repo_id)
                        .bind(limit)
                        .bind(offset)
                        .fetch_all(p)
                        .await
                }
            }
            .map_err(|e| format!("list issues failed: {e}"))?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(map_issue!(&r));
            }
            Ok((out, total))
        }
        DbPool::Sqlite(p) => {
            let total: i64 = match state {
                Some(s) => sqlx::query_scalar(
                    "SELECT COUNT(*) FROM issues WHERE repo_id = ?1 AND state = ?2",
                )
                .bind(repo_id)
                .bind(s)
                .fetch_one(p)
                .await
                .map_err(|e| format!("count issues failed: {e}"))?,
                None => sqlx::query_scalar("SELECT COUNT(*) FROM issues WHERE repo_id = ?1")
                    .bind(repo_id)
                    .fetch_one(p)
                    .await
                    .map_err(|e| format!("count issues failed: {e}"))?,
            };
            let sql = match state {
                Some(_) => format!(
                    "{ISSUE_SELECT_SQLITE} WHERE repo_id = ?1 AND state = ?2
 ORDER BY updated_at DESC LIMIT ?3 OFFSET ?4"
                ),
                None => format!(
                    "{ISSUE_SELECT_SQLITE} WHERE repo_id = ?1
 ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3"
                ),
            };
            let rows = match state {
                Some(s) => {
                    sqlx::query(&sql)
                        .bind(repo_id)
                        .bind(s)
                        .bind(limit)
                        .bind(offset)
                        .fetch_all(p)
                        .await
                }
                None => {
                    sqlx::query(&sql)
                        .bind(repo_id)
                        .bind(limit)
                        .bind(offset)
                        .fetch_all(p)
                        .await
                }
            }
            .map_err(|e| format!("list issues failed: {e}"))?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(map_issue!(&r));
            }
            Ok((out, total))
        }
    }
}

/// Hard-delete an issue. Does **not** decrement `issue_counters` (D-ISS-01 / D-ISS-02).
pub async fn delete_issue(pool: &DbPool, id: &str) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query("DELETE FROM issues WHERE id = $1")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("delete issue failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query("DELETE FROM issues WHERE id = ?")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("delete issue failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query("DELETE FROM issues WHERE id = ?1")
                .bind(id)
                .execute(p)
                .await
                .map_err(|e| format!("delete issue failed: {e}"))?;
        }
    }
    Ok(())
}
