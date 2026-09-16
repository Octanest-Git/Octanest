//! Actions runners / runs / jobs / secrets persistence (Phase 19 / D-ACT-13 / D-ACT-17).

use crate::pool::DbPool;
use sqlx::Row;

#[derive(Debug, Clone)]
pub struct ActionRunnerRow {
    pub id: String,
    pub name: String,
    pub token_hash: String,
    pub labels_json: String,
    pub owner_type: Option<String>,
    pub owner_id: Option<String>,
    pub repository_id: Option<String>,
    pub ephemeral: bool,
    pub last_online: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ActionRunRow {
    pub id: String,
    pub repository_id: String,
    pub workflow_path: String,
    pub workflow_name: String,
    pub event: String,
    pub head_sha: String,
    pub head_ref: String,
    pub status: String,
    pub title: String,
    pub triggered_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActionJobRow {
    pub id: String,
    pub run_id: String,
    pub job_key: String,
    pub name: String,
    pub runs_on_json: String,
    pub status: String,
    pub runner_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ActionSecretMetaRow {
    pub id: String,
    pub repository_id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn insert_runner(
    pool: &DbPool,
    id: &str,
    name: &str,
    token_hash: &str,
    labels_json: &str,
    repository_id: Option<&str>,
    ephemeral: bool,
) -> Result<ActionRunnerRow, String> {
    let eph_i = if ephemeral { 1i64 } else { 0 };
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                r#"INSERT INTO action_runners
                   (id, name, token_hash, labels_json, repository_id, ephemeral)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(id).bind(name).bind(token_hash).bind(labels_json).bind(repository_id).bind(ephemeral)
            .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::MySql(p) => {
            sqlx::query(
                r#"INSERT INTO action_runners
                   (id, name, token_hash, labels_json, repository_id, ephemeral)
                   VALUES (?, ?, ?, ?, ?, ?)"#,
            )
            .bind(id).bind(name).bind(token_hash).bind(labels_json).bind(repository_id).bind(eph_i as i8)
            .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                r#"INSERT INTO action_runners
                   (id, name, token_hash, labels_json, repository_id, ephemeral)
                   VALUES (?, ?, ?, ?, ?, ?)"#,
            )
            .bind(id).bind(name).bind(token_hash).bind(labels_json).bind(repository_id).bind(eph_i)
            .execute(p).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(ActionRunnerRow {
        id: id.into(), name: name.into(), token_hash: token_hash.into(),
        labels_json: labels_json.into(), owner_type: None, owner_id: None,
        repository_id: repository_id.map(str::to_string), ephemeral,
        last_online: None, created_at: String::new(), updated_at: String::new(),
    })
}

pub async fn find_runner_by_id(
    pool: &DbPool,
    id: &str,
) -> Result<Option<ActionRunnerRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query(
                r#"SELECT id, name, token_hash, labels_json, owner_type, owner_id, repository_id, ephemeral,
                          to_char(last_online AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_online,
                          to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
                          to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
                   FROM action_runners WHERE id = $1"#,
            ).bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionRunnerRow {
                id: r.get("id"), name: r.get("name"), token_hash: r.get("token_hash"),
                labels_json: r.get("labels_json"), owner_type: r.get("owner_type"),
                owner_id: r.get("owner_id"), repository_id: r.get("repository_id"),
                ephemeral: r.get("ephemeral"), last_online: r.get("last_online"),
                created_at: r.try_get("created_at").unwrap_or_default(),
                updated_at: r.try_get("updated_at").unwrap_or_default(),
            }))
        }
        DbPool::MySql(p) => {
            let row = sqlx::query(
                r#"SELECT id, name, token_hash, labels_json, owner_type, owner_id, repository_id, ephemeral,
                          DATE_FORMAT(last_online, '%Y-%m-%dT%H:%i:%sZ') AS last_online,
                          DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
                          DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
                   FROM action_runners WHERE id = ?"#,
            ).bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionRunnerRow {
                id: r.get("id"), name: r.get("name"), token_hash: r.get("token_hash"),
                labels_json: r.get("labels_json"), owner_type: r.get("owner_type"),
                owner_id: r.get("owner_id"), repository_id: r.get("repository_id"),
                ephemeral: r.try_get::<i8,_>("ephemeral").map(|i| i!=0).unwrap_or(false),
                last_online: r.get("last_online"),
                created_at: r.try_get("created_at").unwrap_or_default(),
                updated_at: r.try_get("updated_at").unwrap_or_default(),
            }))
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query(
                r#"SELECT id, name, token_hash, labels_json, owner_type, owner_id, repository_id,
                          ephemeral, last_online,
                          strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
                          strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
                   FROM action_runners WHERE id = ?"#,
            ).bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionRunnerRow {
                id: r.get("id"), name: r.get("name"), token_hash: r.get("token_hash"),
                labels_json: r.get("labels_json"), owner_type: r.get("owner_type"),
                owner_id: r.get("owner_id"), repository_id: r.get("repository_id"),
                ephemeral: r.try_get::<i64,_>("ephemeral").map(|i| i!=0).unwrap_or(false),
                last_online: r.get("last_online"),
                created_at: r.try_get("created_at").unwrap_or_default(),
                updated_at: r.try_get("updated_at").unwrap_or_default(),
            }))
        }
    }
}

pub async fn insert_run(
    pool: &DbPool, id: &str, repository_id: &str, workflow_path: &str, workflow_name: &str,
    event: &str, head_sha: &str, head_ref: &str, title: &str, triggered_by: Option<&str>,
) -> Result<ActionRunRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(r#"INSERT INTO action_runs
                (id, repository_id, workflow_path, workflow_name, event, head_sha, head_ref, title, triggered_by)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#)
                .bind(id).bind(repository_id).bind(workflow_path).bind(workflow_name)
                .bind(event).bind(head_sha).bind(head_ref).bind(title).bind(triggered_by)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::MySql(p) => {
            sqlx::query(r#"INSERT INTO action_runs
                (id, repository_id, workflow_path, workflow_name, event, head_sha, head_ref, title, triggered_by)
                VALUES (?,?,?,?,?,?,?,?,?)"#)
                .bind(id).bind(repository_id).bind(workflow_path).bind(workflow_name)
                .bind(event).bind(head_sha).bind(head_ref).bind(title).bind(triggered_by)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(r#"INSERT INTO action_runs
                (id, repository_id, workflow_path, workflow_name, event, head_sha, head_ref, title, triggered_by)
                VALUES (?,?,?,?,?,?,?,?,?)"#)
                .bind(id).bind(repository_id).bind(workflow_path).bind(workflow_name)
                .bind(event).bind(head_sha).bind(head_ref).bind(title).bind(triggered_by)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(ActionRunRow {
        id: id.into(), repository_id: repository_id.into(), workflow_path: workflow_path.into(),
        workflow_name: workflow_name.into(), event: event.into(), head_sha: head_sha.into(),
        head_ref: head_ref.into(), status: "queued".into(), title: title.into(),
        triggered_by: triggered_by.map(str::to_string), created_at: String::new(),
        updated_at: String::new(), finished_at: None,
    })
}

pub async fn find_run_by_id(pool: &DbPool, id: &str) -> Result<Option<ActionRunRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query("SELECT id, repository_id, workflow_path, workflow_name, event, head_sha, head_ref, status, title, triggered_by FROM action_runs WHERE id = $1")
                .bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionRunRow {
                id: r.get("id"),
                repository_id: r.get("repository_id"),
                workflow_path: r.get("workflow_path"),
                workflow_name: r.get("workflow_name"),
                event: r.get("event"),
                head_sha: r.get("head_sha"),
                head_ref: r.get("head_ref"),
                status: r.get("status"),
                title: r.get("title"),
                triggered_by: r.get("triggered_by"),
                created_at: String::new(),
                updated_at: String::new(),
                finished_at: None,
            }))
        }
        DbPool::MySql(p) => {
            let row = sqlx::query("SELECT id, repository_id, workflow_path, workflow_name, event, head_sha, head_ref, status, title, triggered_by FROM action_runs WHERE id = ?")
                .bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionRunRow {
                id: r.get("id"),
                repository_id: r.get("repository_id"),
                workflow_path: r.get("workflow_path"),
                workflow_name: r.get("workflow_name"),
                event: r.get("event"),
                head_sha: r.get("head_sha"),
                head_ref: r.get("head_ref"),
                status: r.get("status"),
                title: r.get("title"),
                triggered_by: r.get("triggered_by"),
                created_at: String::new(),
                updated_at: String::new(),
                finished_at: None,
            }))
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query("SELECT id, repository_id, workflow_path, workflow_name, event, head_sha, head_ref, status, title, triggered_by FROM action_runs WHERE id = ?")
                .bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionRunRow {
                id: r.get("id"),
                repository_id: r.get("repository_id"),
                workflow_path: r.get("workflow_path"),
                workflow_name: r.get("workflow_name"),
                event: r.get("event"),
                head_sha: r.get("head_sha"),
                head_ref: r.get("head_ref"),
                status: r.get("status"),
                title: r.get("title"),
                triggered_by: r.get("triggered_by"),
                created_at: String::new(),
                updated_at: String::new(),
                finished_at: None,
            }))
        }
    }
}

pub async fn insert_job(
    pool: &DbPool, id: &str, run_id: &str, job_key: &str, name: &str, runs_on_json: &str,
) -> Result<ActionJobRow, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(r#"INSERT INTO action_jobs (id, run_id, job_key, name, runs_on_json) VALUES ($1,$2,$3,$4,$5)"#)
                .bind(id).bind(run_id).bind(job_key).bind(name).bind(runs_on_json)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::MySql(p) => {
            sqlx::query(r#"INSERT INTO action_jobs (id, run_id, job_key, name, runs_on_json) VALUES (?,?,?,?,?)"#)
                .bind(id).bind(run_id).bind(job_key).bind(name).bind(runs_on_json)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(r#"INSERT INTO action_jobs (id, run_id, job_key, name, runs_on_json) VALUES (?,?,?,?,?)"#)
                .bind(id).bind(run_id).bind(job_key).bind(name).bind(runs_on_json)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(ActionJobRow {
        id: id.into(), run_id: run_id.into(), job_key: job_key.into(), name: name.into(),
        runs_on_json: runs_on_json.into(), status: "queued".into(), runner_id: None,
        started_at: None, finished_at: None, created_at: String::new(), updated_at: String::new(),
    })
}

pub async fn find_job_by_id(pool: &DbPool, id: &str) -> Result<Option<ActionJobRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let row = sqlx::query("SELECT id, run_id, job_key, name, runs_on_json, status, runner_id FROM action_jobs WHERE id = $1")
                .bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionJobRow {
                id: r.get("id"),
                run_id: r.get("run_id"),
                job_key: r.get("job_key"),
                name: r.get("name"),
                runs_on_json: r.get("runs_on_json"),
                status: r.get("status"),
                runner_id: r.get("runner_id"),
                started_at: None,
                finished_at: None,
                created_at: String::new(),
                updated_at: String::new(),
            }))
        }
        DbPool::MySql(p) => {
            let row = sqlx::query("SELECT id, run_id, job_key, name, runs_on_json, status, runner_id FROM action_jobs WHERE id = ?")
                .bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionJobRow {
                id: r.get("id"),
                run_id: r.get("run_id"),
                job_key: r.get("job_key"),
                name: r.get("name"),
                runs_on_json: r.get("runs_on_json"),
                status: r.get("status"),
                runner_id: r.get("runner_id"),
                started_at: None,
                finished_at: None,
                created_at: String::new(),
                updated_at: String::new(),
            }))
        }
        DbPool::Sqlite(p) => {
            let row = sqlx::query("SELECT id, run_id, job_key, name, runs_on_json, status, runner_id FROM action_jobs WHERE id = ?")
                .bind(id).fetch_optional(p).await.map_err(|e| e.to_string())?;
            Ok(row.map(|r| ActionJobRow {
                id: r.get("id"),
                run_id: r.get("run_id"),
                job_key: r.get("job_key"),
                name: r.get("name"),
                runs_on_json: r.get("runs_on_json"),
                status: r.get("status"),
                runner_id: r.get("runner_id"),
                started_at: None,
                finished_at: None,
                created_at: String::new(),
                updated_at: String::new(),
            }))
        }
    }
}

pub async fn insert_secret(
    pool: &DbPool, id: &str, repository_id: &str, name: &str, ciphertext: &str,
) -> Result<(), String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(r#"INSERT INTO action_secrets (id, repository_id, name, ciphertext) VALUES ($1,$2,$3,$4)"#)
                .bind(id).bind(repository_id).bind(name).bind(ciphertext)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::MySql(p) => {
            sqlx::query(r#"INSERT INTO action_secrets (id, repository_id, name, ciphertext) VALUES (?,?,?,?)"#)
                .bind(id).bind(repository_id).bind(name).bind(ciphertext)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(r#"INSERT INTO action_secrets (id, repository_id, name, ciphertext) VALUES (?,?,?,?)"#)
                .bind(id).bind(repository_id).bind(name).bind(ciphertext)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub async fn list_secret_names(
    pool: &DbPool, repository_id: &str,
) -> Result<Vec<ActionSecretMetaRow>, String> {
    match pool {
        DbPool::Postgres(p) => {
            let rows = sqlx::query(
                r#"SELECT id, repository_id, name,
                          to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
                          to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
                   FROM action_secrets WHERE repository_id = $1 ORDER BY name"#,
            ).bind(repository_id).fetch_all(p).await.map_err(|e| e.to_string())?;
            Ok(rows.iter().map(|row| ActionSecretMetaRow {
                id: row.get("id"), repository_id: row.get("repository_id"), name: row.get("name"),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            }).collect())
        }
        DbPool::MySql(p) => {
            let rows = sqlx::query(
                r#"SELECT id, repository_id, name,
                          DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS created_at,
                          DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%sZ') AS updated_at
                   FROM action_secrets WHERE repository_id = ? ORDER BY name"#,
            ).bind(repository_id).fetch_all(p).await.map_err(|e| e.to_string())?;
            Ok(rows.iter().map(|row| ActionSecretMetaRow {
                id: row.get("id"), repository_id: row.get("repository_id"), name: row.get("name"),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            }).collect())
        }
        DbPool::Sqlite(p) => {
            let rows = sqlx::query(
                r#"SELECT id, repository_id, name,
                          strftime('%Y-%m-%dT%H:%M:%SZ', created_at) AS created_at,
                          strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) AS updated_at
                   FROM action_secrets WHERE repository_id = ? ORDER BY name"#,
            ).bind(repository_id).fetch_all(p).await.map_err(|e| e.to_string())?;
            Ok(rows.iter().map(|row| ActionSecretMetaRow {
                id: row.get("id"), repository_id: row.get("repository_id"), name: row.get("name"),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            }).collect())
        }
    }
}

pub async fn insert_runner_token(
    pool: &DbPool, id: &str, token_hash: &str, scope_type: &str, scope_id: Option<&str>, active: bool,
) -> Result<(), String> {
    let active_i = if active { 1i64 } else { 0 };
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(r#"INSERT INTO action_runner_tokens (id, token_hash, scope_type, scope_id, active) VALUES ($1,$2,$3,$4,$5)"#)
                .bind(id).bind(token_hash).bind(scope_type).bind(scope_id).bind(active)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::MySql(p) => {
            sqlx::query(r#"INSERT INTO action_runner_tokens (id, token_hash, scope_type, scope_id, active) VALUES (?,?,?,?,?)"#)
                .bind(id).bind(token_hash).bind(scope_type).bind(scope_id).bind(active_i as i8)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
        DbPool::Sqlite(p) => {
            sqlx::query(r#"INSERT INTO action_runner_tokens (id, token_hash, scope_type, scope_id, active) VALUES (?,?,?,?,?)"#)
                .bind(id).bind(token_hash).bind(scope_type).bind(scope_id).bind(active_i)
                .execute(p).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub async fn wipe_actions_domain(pool: &DbPool) -> Result<(), String> {
    for sql in [
        "DELETE FROM action_jobs",
        "DELETE FROM action_runs",
        "DELETE FROM action_secrets",
        "DELETE FROM action_runners",
        "DELETE FROM action_runner_tokens",
    ] {
        match pool {
            DbPool::Postgres(p) => { sqlx::query(sql).execute(p).await.map_err(|e| e.to_string())?; }
            DbPool::MySql(p) => { sqlx::query(sql).execute(p).await.map_err(|e| e.to_string())?; }
            DbPool::Sqlite(p) => { sqlx::query(sql).execute(p).await.map_err(|e| e.to_string())?; }
        }
    }
    Ok(())
}
