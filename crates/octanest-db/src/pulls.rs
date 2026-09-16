//! Pull request persistence — dialect SQL via `DbPool` match (Phase 12).

use sqlx::Row;

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct PullRow {
    pub id: String,
    pub repo_id: String,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub draft: bool,
    pub author_id: String,
    pub base_ref: String,
    pub base_sha: String,
    pub head_repo_id: String,
    pub head_ref: String,
    pub head_sha: String,
    pub merged_at: Option<String>,
    pub merged_by: Option<String>,
    pub merge_commit_sha: Option<String>,
    pub merge_method: Option<String>,
    pub closed_at: Option<String>,
    pub closed_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct RepoMergeSettingsRow {
    pub allow_merge_commit: bool,
    pub allow_squash_merge: bool,
    pub allow_rebase_merge: bool,
}

macro_rules! flag_col {
    ($row:expr, $col:expr) => {{
        match $row.try_get::<bool, _>($col) {
            Ok(v) => v,
            Err(_) => {
                let n: i64 = $row
                    .try_get($col)
                    .map_err(|e| format!("pull row {}: {e}", $col))?;
                n != 0
            }
        }
    }};
}

macro_rules! map_pull {
    ($row:expr) => {{
        let row = $row;
        PullRow {
            id: row.try_get("id").map_err(|e| format!("pull row: {e}"))?,
            repo_id: row.try_get("repo_id").map_err(|e| format!("pull row: {e}"))?,
            number: row.try_get("number").map_err(|e| format!("pull row: {e}"))?,
            title: row.try_get("title").map_err(|e| format!("pull row: {e}"))?,
            body: row.try_get("body").map_err(|e| format!("pull row: {e}"))?,
            state: row.try_get("state").map_err(|e| format!("pull row: {e}"))?,
            draft: flag_col!(row, "draft"),
            author_id: row
                .try_get("author_id")
                .map_err(|e| format!("pull row: {e}"))?,
            base_ref: row.try_get("base_ref").map_err(|e| format!("pull row: {e}"))?,
            base_sha: row.try_get("base_sha").map_err(|e| format!("pull row: {e}"))?,
            head_repo_id: row
                .try_get("head_repo_id")
                .map_err(|e| format!("pull row: {e}"))?,
            head_ref: row.try_get("head_ref").map_err(|e| format!("pull row: {e}"))?,
            head_sha: row.try_get("head_sha").map_err(|e| format!("pull row: {e}"))?,
            merged_at: row.try_get("merged_at").map_err(|e| format!("pull row: {e}"))?,
            merged_by: row.try_get("merged_by").map_err(|e| format!("pull row: {e}"))?,
            merge_commit_sha: row
                .try_get("merge_commit_sha")
                .map_err(|e| format!("pull row: {e}"))?,
            merge_method: row
                .try_get("merge_method")
                .map_err(|e| format!("pull row: {e}"))?,
            closed_at: row.try_get("closed_at").map_err(|e| format!("pull row: {e}"))?,
            closed_by: row.try_get("closed_by").map_err(|e| format!("pull row: {e}"))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("pull row: {e}"))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("pull row: {e}"))?,
        }
    }};
}

const PULL_COLS: &str = "id, repo_id, number, title, body, state, draft, author_id, base_ref, base_sha, head_repo_id, head_ref, head_sha, merged_at, merged_by, merge_commit_sha, merge_method, closed_at, closed_by, created_at, updated_at";

pub async fn insert_pull(
    pool: &DbPool,
    id: &str,
    repo_id: &str,
    number: i64,
    title: &str,
    body: &str,
    author_id: &str,
    base_ref: &str,
    base_sha: &str,
    head_repo_id: &str,
    head_ref: &str,
    head_sha: &str,
    draft: bool,
) -> Result<PullRow, String> {
    let draft_i: i64 = if draft { 1 } else { 0 };
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO pull_requests (
                  id, repo_id, number, title, body, author_id, base_ref, base_sha,
                  head_repo_id, head_ref, head_sha, draft
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
            )
            .bind(id)
            .bind(repo_id)
            .bind(number)
            .bind(title)
            .bind(body)
            .bind(author_id)
            .bind(base_ref)
            .bind(base_sha)
            .bind(head_repo_id)
            .bind(head_ref)
            .bind(head_sha)
            .bind(draft)
            .execute(p)
            .await
            .map_err(|e| format!("insert pull failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO pull_requests (
                  id, repo_id, number, title, body, author_id, base_ref, base_sha,
                  head_repo_id, head_ref, head_sha, draft
                ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
            )
            .bind(id)
            .bind(repo_id)
            .bind(number)
            .bind(title)
            .bind(body)
            .bind(author_id)
            .bind(base_ref)
            .bind(base_sha)
            .bind(head_repo_id)
            .bind(head_ref)
            .bind(head_sha)
            .bind(draft_i)
            .execute(p)
            .await
            .map_err(|e| format!("insert pull failed: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO pull_requests (
                  id, repo_id, number, title, body, author_id, base_ref, base_sha,
                  head_repo_id, head_ref, head_sha, draft
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            )
            .bind(id)
            .bind(repo_id)
            .bind(number)
            .bind(title)
            .bind(body)
            .bind(author_id)
            .bind(base_ref)
            .bind(base_sha)
            .bind(head_repo_id)
            .bind(head_ref)
            .bind(head_sha)
            .bind(draft_i)
            .execute(p)
            .await
            .map_err(|e| format!("insert pull failed: {e}"))?;
        }
    }
    find_by_repo_and_number(pool, repo_id, number)
        .await?
        .ok_or_else(|| "insert pull failed: row missing after insert".into())
}

pub async fn find_by_repo_and_number(
    pool: &DbPool,
    repo_id: &str,
    number: i64,
) -> Result<Option<PullRow>, String> {
    let sql = format!("SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = $1 AND number = $2");
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(&sql)
                .bind(repo_id)
                .bind(number)
                .fetch_optional(p)
                .await
                .map_err(|e| format!("find pull failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_pull!(&r)),
                None => None,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(&format!(
                "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = ? AND number = ?"
            ))
            .bind(repo_id)
            .bind(number)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find pull failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_pull!(&r)),
                None => None,
            })
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(&format!(
                "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = ?1 AND number = ?2"
            ))
            .bind(repo_id)
            .bind(number)
            .fetch_optional(p)
            .await
            .map_err(|e| format!("find pull failed: {e}"))?;
            Ok(match row {
                Some(r) => Some(map_pull!(&r)),
                None => None,
            })
        }
    }
}

pub async fn list_by_repo(
    pool: &DbPool,
    repo_id: &str,
    state: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<(Vec<PullRow>, i64), String> {
    let limit = limit.clamp(1, 100);
    match pool {
        DbPool::Sqlite(p) => {
            let (rows, total) = if let Some(st) = state.filter(|s| *s != "all") {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM pull_requests WHERE repo_id = ?1 AND state = ?2",
                )
                .bind(repo_id)
                .bind(st)
                .fetch_one(p)
                .await
                .map_err(|e| format!("count pulls failed: {e}"))?;
                let rows = sqlx::query(&format!(
                    "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = ?1 AND state = ?2
                     ORDER BY updated_at DESC LIMIT ?3 OFFSET ?4"
                ))
                .bind(repo_id)
                .bind(st)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list pulls failed: {e}"))?;
                (rows, total)
            } else {
                let total: i64 =
                    sqlx::query_scalar("SELECT COUNT(*) FROM pull_requests WHERE repo_id = ?1")
                        .bind(repo_id)
                        .fetch_one(p)
                        .await
                        .map_err(|e| format!("count pulls failed: {e}"))?;
                let rows = sqlx::query(&format!(
                    "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = ?1
                     ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3"
                ))
                .bind(repo_id)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list pulls failed: {e}"))?;
                (rows, total)
            };
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(map_pull!(&r));
            }
            Ok((out, total))
        }
        DbPool::Postgres(p) => {
            let (rows, total) = if let Some(st) = state.filter(|s| *s != "all") {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1 AND state = $2",
                )
                .bind(repo_id)
                .bind(st)
                .fetch_one(p)
                .await
                .map_err(|e| format!("count pulls failed: {e}"))?;
                let rows = sqlx::query(&format!(
                    "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = $1 AND state = $2
                     ORDER BY updated_at DESC LIMIT $3 OFFSET $4"
                ))
                .bind(repo_id)
                .bind(st)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list pulls failed: {e}"))?;
                (rows, total)
            } else {
                let total: i64 =
                    sqlx::query_scalar("SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1")
                        .bind(repo_id)
                        .fetch_one(p)
                        .await
                        .map_err(|e| format!("count pulls failed: {e}"))?;
                let rows = sqlx::query(&format!(
                    "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = $1
                     ORDER BY updated_at DESC LIMIT $2 OFFSET $3"
                ))
                .bind(repo_id)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list pulls failed: {e}"))?;
                (rows, total)
            };
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(map_pull!(&r));
            }
            Ok((out, total))
        }
        DbPool::MySql(p) => {
            let (rows, total) = if let Some(st) = state.filter(|s| *s != "all") {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM pull_requests WHERE repo_id = ? AND state = ?",
                )
                .bind(repo_id)
                .bind(st)
                .fetch_one(p)
                .await
                .map_err(|e| format!("count pulls failed: {e}"))?;
                let rows = sqlx::query(&format!(
                    "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = ? AND state = ?
                     ORDER BY updated_at DESC LIMIT ? OFFSET ?"
                ))
                .bind(repo_id)
                .bind(st)
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list pulls failed: {e}"))?;
                (rows, total)
            } else {
                let total: i64 =
                    sqlx::query_scalar("SELECT COUNT(*) FROM pull_requests WHERE repo_id = ?")
                        .bind(repo_id)
                        .fetch_one(p)
                        .await
                        .map_err(|e| format!("count pulls failed: {e}"))?;
                let rows = sqlx::query(&format!(
                    "SELECT {PULL_COLS} FROM pull_requests WHERE repo_id = ?
                     ORDER BY updated_at DESC LIMIT ? OFFSET ?"
                ))
                .bind(repo_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(p)
                .await
                .map_err(|e| format!("list pulls failed: {e}"))?;
                (rows, total)
            };
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(map_pull!(&r));
            }
            Ok((out, total))
        }
    }
}

pub async fn set_state(
    pool: &DbPool,
    id: &str,
    state: &str,
    closed_at: Option<&str>,
    closed_by: Option<&str>,
) -> Result<(), String> {
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE pull_requests SET state = ?1, closed_at = ?2, closed_by = ?3,
                 updated_at = strftime('%Y-%m-%d %H:%M:%S','now') WHERE id = ?4",
            )
            .bind(state)
            .bind(closed_at)
            .bind(closed_by)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("set pull state failed: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE pull_requests SET state = $1, closed_at = $2, closed_by = $3,
                 updated_at = now() WHERE id = $4",
            )
            .bind(state)
            .bind(closed_at)
            .bind(closed_by)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("set pull state failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE pull_requests SET state = ?, closed_at = ?, closed_by = ?,
                 updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(state)
            .bind(closed_at)
            .bind(closed_by)
            .bind(id)
            .execute(p)
            .await
            .map_err(|e| format!("set pull state failed: {e}"))?;
        }
    }
    Ok(())
}

pub async fn get_merge_settings(
    pool: &DbPool,
    repo_id: &str,
) -> Result<RepoMergeSettingsRow, String> {
    match pool {
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                "SELECT allow_merge_commit, allow_squash_merge, allow_rebase_merge
                 FROM repositories WHERE id = ?1",
            )
            .bind(repo_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("get merge settings failed: {e}"))?;
            Ok(RepoMergeSettingsRow {
                allow_merge_commit: flag_col!(row, "allow_merge_commit"),
                allow_squash_merge: flag_col!(row, "allow_squash_merge"),
                allow_rebase_merge: flag_col!(row, "allow_rebase_merge"),
            })
        }
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                "SELECT allow_merge_commit, allow_squash_merge, allow_rebase_merge
                 FROM repositories WHERE id = $1",
            )
            .bind(repo_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("get merge settings failed: {e}"))?;
            Ok(RepoMergeSettingsRow {
                allow_merge_commit: row
                    .try_get("allow_merge_commit")
                    .map_err(|e| format!("merge settings: {e}"))?,
                allow_squash_merge: row
                    .try_get("allow_squash_merge")
                    .map_err(|e| format!("merge settings: {e}"))?,
                allow_rebase_merge: row
                    .try_get("allow_rebase_merge")
                    .map_err(|e| format!("merge settings: {e}"))?,
            })
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                "SELECT allow_merge_commit, allow_squash_merge, allow_rebase_merge
                 FROM repositories WHERE id = ?",
            )
            .bind(repo_id)
            .fetch_one(p)
            .await
            .map_err(|e| format!("get merge settings failed: {e}"))?;
            Ok(RepoMergeSettingsRow {
                allow_merge_commit: flag_col!(row, "allow_merge_commit"),
                allow_squash_merge: flag_col!(row, "allow_squash_merge"),
                allow_rebase_merge: flag_col!(row, "allow_rebase_merge"),
            })
        }
    }
}

pub async fn set_merge_settings(
    pool: &DbPool,
    repo_id: &str,
    allow_merge_commit: bool,
    allow_squash_merge: bool,
    allow_rebase_merge: bool,
) -> Result<(), String> {
    let a = if allow_merge_commit { 1 } else { 0 };
    let b = if allow_squash_merge { 1 } else { 0 };
    let c = if allow_rebase_merge { 1 } else { 0 };
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query(
                "UPDATE repositories SET allow_merge_commit = ?1, allow_squash_merge = ?2,
                 allow_rebase_merge = ?3 WHERE id = ?4",
            )
            .bind(a)
            .bind(b)
            .bind(c)
            .bind(repo_id)
            .execute(p)
            .await
            .map_err(|e| format!("set merge settings failed: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query(
                "UPDATE repositories SET allow_merge_commit = $1, allow_squash_merge = $2,
                 allow_rebase_merge = $3 WHERE id = $4",
            )
            .bind(allow_merge_commit)
            .bind(allow_squash_merge)
            .bind(allow_rebase_merge)
            .bind(repo_id)
            .execute(p)
            .await
            .map_err(|e| format!("set merge settings failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "UPDATE repositories SET allow_merge_commit = ?, allow_squash_merge = ?,
                 allow_rebase_merge = ? WHERE id = ?",
            )
            .bind(a)
            .bind(b)
            .bind(c)
            .bind(repo_id)
            .execute(p)
            .await
            .map_err(|e| format!("set merge settings failed: {e}"))?;
        }
    }
    Ok(())
}

pub async fn set_forked_from(
    pool: &DbPool,
    repo_id: &str,
    forked_from: Option<&str>,
) -> Result<(), String> {
    match pool {
        DbPool::Sqlite(p) => {
            sqlx::query("UPDATE repositories SET forked_from_repo_id = ?1 WHERE id = ?2")
                .bind(forked_from)
                .bind(repo_id)
                .execute(p)
                .await
                .map_err(|e| format!("set forked_from failed: {e}"))?;
        }
        DbPool::Postgres(p) => {
            sqlx::query("UPDATE repositories SET forked_from_repo_id = $1 WHERE id = $2")
                .bind(forked_from)
                .bind(repo_id)
                .execute(p)
                .await
                .map_err(|e| format!("set forked_from failed: {e}"))?;
        }
        DbPool::MySql(p) => {
            sqlx::query("UPDATE repositories SET forked_from_repo_id = ? WHERE id = ?")
                .bind(forked_from)
                .bind(repo_id)
                .execute(p)
                .await
                .map_err(|e| format!("set forked_from failed: {e}"))?;
        }
    }
    Ok(())
}

pub async fn get_forked_from(pool: &DbPool, repo_id: &str) -> Result<Option<String>, String> {
    match pool {
        DbPool::Sqlite(p) => sqlx::query_scalar(
            "SELECT forked_from_repo_id FROM repositories WHERE id = ?1",
        )
        .bind(repo_id)
        .fetch_one(p)
        .await
        .map_err(|e| format!("get forked_from failed: {e}")),
        DbPool::Postgres(p) => sqlx::query_scalar(
            "SELECT forked_from_repo_id FROM repositories WHERE id = $1",
        )
        .bind(repo_id)
        .fetch_one(p)
        .await
        .map_err(|e| format!("get forked_from failed: {e}")),
        DbPool::MySql(p) => {
            sqlx::query_scalar("SELECT forked_from_repo_id FROM repositories WHERE id = ?")
                .bind(repo_id)
                .fetch_one(p)
                .await
                .map_err(|e| format!("get forked_from failed: {e}"))
        }
    }
}
