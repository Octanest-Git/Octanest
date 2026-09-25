//! Repository topics (issue #23).

use crate::pool::DbPool;

/// Max topics linked to one repository.
pub const MAX_REPO_TOPICS: usize = 20;

/// Normalize + validate a topic slug: trim, lowercase, `[a-z0-9]([a-z0-9-]{0,49})`.
pub fn normalize_topic_name(raw: &str) -> Result<String, String> {
    let name = raw.trim().to_lowercase();
    if name.is_empty() {
        return Err("topic name must not be empty".into());
    }
    if name.len() > 50 {
        return Err("topic name must be at most 50 characters".into());
    }
    let bytes = name.as_bytes();
    let first = bytes[0];
    if !first.is_ascii_alphanumeric() {
        return Err("topic name must start with a letter or digit".into());
    }
    for &b in &bytes[1..] {
        if !(b.is_ascii_alphanumeric() || b == b'-') {
            return Err("topic name may only contain letters, digits, and hyphens".into());
        }
    }
    Ok(name)
}

fn normalize_topic_list(topics: &[String]) -> Result<Vec<String>, String> {
    if topics.len() > MAX_REPO_TOPICS {
        return Err(format!("at most {MAX_REPO_TOPICS} topics allowed"));
    }
    let mut out = Vec::with_capacity(topics.len());
    for t in topics {
        let n = normalize_topic_name(t)?;
        if !out.iter().any(|x| x == &n) {
            out.push(n);
        }
    }
    if out.len() > MAX_REPO_TOPICS {
        return Err(format!("at most {MAX_REPO_TOPICS} topics allowed"));
    }
    Ok(out)
}

/// Topic names for a repository, ordered by name.
pub async fn list_repo_topics(pool: &DbPool, repo_id: &str) -> Result<Vec<String>, String> {
    match pool {
        DbPool::Postgres(p) => sqlx::query_scalar(
            "SELECT t.name FROM repository_topics rt
             JOIN topics t ON t.id = rt.topic_id
             WHERE rt.repository_id = $1
             ORDER BY t.name ASC",
        )
        .bind(repo_id)
        .fetch_all(p)
        .await
        .map_err(|e| format!("list_repo_topics: {e}")),
        DbPool::MySql(p) => sqlx::query_scalar(
            "SELECT t.name FROM repository_topics rt
             JOIN topics t ON t.id = rt.topic_id
             WHERE rt.repository_id = ?
             ORDER BY t.name ASC",
        )
        .bind(repo_id)
        .fetch_all(p)
        .await
        .map_err(|e| format!("list_repo_topics: {e}")),
        DbPool::Sqlite(p) => sqlx::query_scalar(
            "SELECT t.name FROM repository_topics rt
             JOIN topics t ON t.id = rt.topic_id
             WHERE rt.repository_id = ?1
             ORDER BY t.name ASC",
        )
        .bind(repo_id)
        .fetch_all(p)
        .await
        .map_err(|e| format!("list_repo_topics: {e}")),
    }
}

async fn upsert_topic_id(pool: &DbPool, name: &str) -> Result<String, String> {
    // Deterministic id from unique slug — avoids a uuid dep in oxidean-db.
    let id = format!("t-{name}");
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO topics (id, name) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
            )
            .bind(&id)
            .bind(name)
            .execute(p)
            .await
            .map_err(|e| format!("upsert topic: {e}"))?;
            let existing: String = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
                .bind(name)
                .fetch_one(p)
                .await
                .map_err(|e| format!("fetch topic id: {e}"))?;
            Ok(existing)
        }
        DbPool::MySql(p) => {
            sqlx::query("INSERT IGNORE INTO topics (id, name) VALUES (?, ?)")
                .bind(&id)
                .bind(name)
                .execute(p)
                .await
                .map_err(|e| format!("upsert topic: {e}"))?;
            let existing: String = sqlx::query_scalar("SELECT id FROM topics WHERE name = ?")
                .bind(name)
                .fetch_one(p)
                .await
                .map_err(|e| format!("fetch topic id: {e}"))?;
            Ok(existing)
        }
        DbPool::Sqlite(p) => {
            sqlx::query("INSERT OR IGNORE INTO topics (id, name) VALUES (?1, ?2)")
                .bind(&id)
                .bind(name)
                .execute(p)
                .await
                .map_err(|e| format!("upsert topic: {e}"))?;
            let existing: String = sqlx::query_scalar("SELECT id FROM topics WHERE name = ?1")
                .bind(name)
                .fetch_one(p)
                .await
                .map_err(|e| format!("fetch topic id: {e}"))?;
            Ok(existing)
        }
    }
}

/// Topic suggestions matching a name prefix, most-used first. Returns
/// `(name, linked_repo_count)` rows for the topics editor autocomplete.
/// Prefix is normalized like a topic slug so LIKE needs no escaping.
pub async fn suggest_topics(
    pool: &DbPool,
    prefix: &str,
    limit: i64,
) -> Result<Vec<(String, i64)>, String> {
    let slug: String = prefix
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(50)
        .collect();
    let limit = limit.clamp(1, 20);
    if slug.is_empty() {
        return match pool {
            DbPool::Postgres(p) => sqlx::query_as::<_, (String, i64)>(
                "SELECT t.name, COUNT(rt.repository_id) AS repo_count
                 FROM topics t
                 LEFT JOIN repository_topics rt ON rt.topic_id = t.id
                 GROUP BY t.id, t.name
                 ORDER BY repo_count DESC, t.name ASC
                 LIMIT $1",
            )
            .bind(limit)
            .fetch_all(p)
            .await
            .map_err(|e| format!("suggest_topics: {e}")),
            DbPool::MySql(p) => sqlx::query_as::<_, (String, i64)>(
                "SELECT t.name, CAST(COUNT(rt.repository_id) AS SIGNED) AS repo_count
                 FROM topics t
                 LEFT JOIN repository_topics rt ON rt.topic_id = t.id
                 GROUP BY t.id, t.name
                 ORDER BY repo_count DESC, t.name ASC
                 LIMIT ?",
            )
            .bind(limit)
            .fetch_all(p)
            .await
            .map_err(|e| format!("suggest_topics: {e}")),
            DbPool::Sqlite(p) => sqlx::query_as::<_, (String, i64)>(
                "SELECT t.name, COUNT(rt.repository_id) AS repo_count
                 FROM topics t
                 LEFT JOIN repository_topics rt ON rt.topic_id = t.id
                 GROUP BY t.id, t.name
                 ORDER BY repo_count DESC, t.name ASC
                 LIMIT ?1",
            )
            .bind(limit)
            .fetch_all(p)
            .await
            .map_err(|e| format!("suggest_topics: {e}")),
        };
    }
    match pool {
        DbPool::Postgres(p) => sqlx::query_as::<_, (String, i64)>(
            "SELECT t.name, COUNT(rt.repository_id) AS repo_count
             FROM topics t
             LEFT JOIN repository_topics rt ON rt.topic_id = t.id
             WHERE t.name LIKE $1 || '%'
             GROUP BY t.id, t.name
             ORDER BY repo_count DESC, t.name ASC
             LIMIT $2",
        )
        .bind(&slug)
        .bind(limit)
        .fetch_all(p)
        .await
        .map_err(|e| format!("suggest_topics: {e}")),
        DbPool::MySql(p) => sqlx::query_as::<_, (String, i64)>(
            "SELECT t.name, CAST(COUNT(rt.repository_id) AS SIGNED) AS repo_count
             FROM topics t
             LEFT JOIN repository_topics rt ON rt.topic_id = t.id
             WHERE t.name LIKE CONCAT(?, '%')
             GROUP BY t.id, t.name
             ORDER BY repo_count DESC, t.name ASC
             LIMIT ?",
        )
        .bind(&slug)
        .bind(limit)
        .fetch_all(p)
        .await
        .map_err(|e| format!("suggest_topics: {e}")),
        DbPool::Sqlite(p) => sqlx::query_as::<_, (String, i64)>(
            "SELECT t.name, COUNT(rt.repository_id) AS repo_count
             FROM topics t
             LEFT JOIN repository_topics rt ON rt.topic_id = t.id
             WHERE t.name LIKE ?1 || '%'
             GROUP BY t.id, t.name
             ORDER BY repo_count DESC, t.name ASC
             LIMIT ?2",
        )
        .bind(&slug)
        .bind(limit)
        .fetch_all(p)
        .await
        .map_err(|e| format!("suggest_topics: {e}")),
    }
}

/// Replace all topic links for a repository. Returns the normalized topic list.
pub async fn set_repo_topics(
    pool: &DbPool,
    repo_id: &str,
    topics: &[String],
) -> Result<Vec<String>, String> {
    let names = normalize_topic_list(topics)?;

    // Resolve / create topic rows first (outside delete) so ids exist.
    let mut topic_ids = Vec::with_capacity(names.len());
    for name in &names {
        topic_ids.push(upsert_topic_id(pool, name).await?);
    }

    match pool {
        DbPool::Postgres(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("set_repo_topics begin: {e}"))?;
            sqlx::query("DELETE FROM repository_topics WHERE repository_id = $1")
                .bind(repo_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear topics: {e}"))?;
            for tid in &topic_ids {
                sqlx::query(
                    "INSERT INTO repository_topics (repository_id, topic_id) VALUES ($1, $2)",
                )
                .bind(repo_id)
                .bind(tid)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("link topic: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("set_repo_topics commit: {e}"))?;
        }
        DbPool::MySql(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("set_repo_topics begin: {e}"))?;
            sqlx::query("DELETE FROM repository_topics WHERE repository_id = ?")
                .bind(repo_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear topics: {e}"))?;
            for tid in &topic_ids {
                sqlx::query(
                    "INSERT INTO repository_topics (repository_id, topic_id) VALUES (?, ?)",
                )
                .bind(repo_id)
                .bind(tid)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("link topic: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("set_repo_topics commit: {e}"))?;
        }
        DbPool::Sqlite(p) => {
            let mut tx = p
                .begin()
                .await
                .map_err(|e| format!("set_repo_topics begin: {e}"))?;
            sqlx::query("DELETE FROM repository_topics WHERE repository_id = ?1")
                .bind(repo_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("clear topics: {e}"))?;
            for tid in &topic_ids {
                sqlx::query(
                    "INSERT INTO repository_topics (repository_id, topic_id) VALUES (?1, ?2)",
                )
                .bind(repo_id)
                .bind(tid)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("link topic: {e}"))?;
            }
            tx.commit()
                .await
                .map_err(|e| format!("set_repo_topics commit: {e}"))?;
        }
    }

    Ok(names)
}
