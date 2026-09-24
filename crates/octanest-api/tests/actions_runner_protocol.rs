//! Phase 19 — runner protocol tracer (ACT-06 / D-ACT-07 / D-ACT-09 / D-ACT-18).

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::actions::dispatch::{dispatch_push_for_sha, enqueue_run};
use octanest_api::actions::mint_registration_token;
use octanest_api::actions::parse_workflow_yaml;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_core::Role;
use octanest_db::Database;
use octanest_git::CliGitBackend;
use tower::ServiceExt;

async fn app_db() -> (axum::Router, Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::connect(&format!("sqlite:{}", dir.path().join("r.db").display()))
        .await
        .unwrap();
    db.migrate().await.unwrap();
    support::unlock_signup(&db).await;
    let state = AppState::new(
        db.clone(),
        Arc::new(LogSink) as Arc<dyn EmailSender>,
        "development",
    )
    .with_actions_enabled(true);
    let app = router_with_state(state, build_cors("development", None).unwrap());
    (app, db, dir)
}

#[tokio::test]
async fn actions_runner_protocol_register() {
    let (app, db, _dir) = app_db().await;
    let reg = mint_registration_token(&db).await.unwrap();
    let body = serde_json::json!({
        "name": "runner-1",
        "labels": ["ubuntu-latest"],
        "token": reg,
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v["runner_id"].as_str().unwrap().len() > 8);
    assert!(v["runner_token"].as_str().unwrap().starts_with("ort_"));
}

#[tokio::test]
async fn actions_runner_protocol_declare_labels() {
    let (app, db, _dir) = app_db().await;
    let reg = mint_registration_token(&db).await.unwrap();
    let reg_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "r1",
                        "labels": ["ubuntu-latest"],
                        "token": reg
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let reg_body = reg_res.into_body().collect().await.unwrap().to_bytes();
    let reg_v: serde_json::Value = serde_json::from_slice(&reg_body).unwrap();
    let runner_token = reg_v["runner_token"].as_str().unwrap();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/declare")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {runner_token}"))
                .body(Body::from(
                    serde_json::json!({
                        "labels": ["ubuntu-latest:docker://node:20", "self-hosted"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn actions_runner_protocol_fetch_task_label_match() {
    let (app, db, dir) = app_db().await;
    let owner = db
        .create_user(
            "u-ft",
            "ft@example.com",
            "ft",
            Some("h"),
            "F",
            "",
            None,
            Role::User,
        )
        .await
        .unwrap();
    let repo = db
        .insert_repository("r-ft", &owner.id, "user", "ft", "private", "", "main")
        .await
        .unwrap();
    let doc = parse_workflow_yaml(
        br#"
name: CI
on: [push]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo hi
"#,
    )
    .unwrap();
    enqueue_run(
        &db,
        &repo.id,
        ".github/workflows/ci.yml",
        &doc,
        "push",
        "deadbeef",
        "refs/heads/main",
        Some(&owner.id),
    )
    .await
    .unwrap();

    let reg = mint_registration_token(&db).await.unwrap();
    let reg_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "r1",
                        "labels": ["ubuntu-latest"],
                        "token": reg
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let reg_body = reg_res.into_body().collect().await.unwrap().to_bytes();
    let reg_v: serde_json::Value = serde_json::from_slice(&reg_body).unwrap();
    let runner_token = reg_v["runner_token"].as_str().unwrap();

    let fetch = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/fetch_task")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {runner_token}"))
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fetch.status(), StatusCode::OK);
    let bytes = fetch.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v["job_id"].as_str().is_some());
    assert_eq!(v["job_key"], "build");
    let _ = dir;
    let _ = dispatch_push_for_sha;
    let _ = CliGitBackend::new();
}

#[tokio::test]
async fn actions_runner_protocol_ignores_session_cookie() {
    let (app, db, _dir) = app_db().await;
    // Signup to get a session cookie
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rpc")
                .header("content-type", "application/json")
                .header("Octanest-RPC-Version", "1")
                .body(Body::from(
                    r#"{"procedure":"auth.signup","input":{"email":"ck@ex.com","username":"ckuser","password":"password1"}}"#,
                ))
                .unwrap(),
        )
        .await;
    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rpc")
                .header("content-type", "application/json")
                .header("Octanest-RPC-Version", "1")
                .body(Body::from(
                    r#"{"procedure":"auth.login","input":{"identifier":"ck@ex.com","password":"password1","remember_me":false}}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let cookie = login
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/fetch_task")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let _ = db;
}

#[tokio::test]
async fn actions_runner_protocol_env_bootstrap_token() {
    let (app, _db, _dir) = app_db().await;
    std::env::set_var(
        "OCTANEST_RUNNER_REGISTRATION_TOKEN",
        "bootstrap-secret-token",
    );
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "ephemeral",
                        "labels": ["ubuntu-latest"],
                        "token": "bootstrap-secret-token"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    std::env::remove_var("OCTANEST_RUNNER_REGISTRATION_TOKEN");
    assert_eq!(res.status(), StatusCode::OK);
}

/// ACT-06: claimed runner can append logs and transition job state via protocol.
#[tokio::test]
async fn actions_runner_protocol_update_task_and_log() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("actions-logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let db = Database::connect(&format!("sqlite:{}", dir.path().join("r.db").display()))
        .await
        .unwrap();
    db.migrate().await.unwrap();
    support::unlock_signup(&db).await;
    let state = AppState::new(
        db.clone(),
        Arc::new(LogSink) as Arc<dyn EmailSender>,
        "development",
    )
    .with_actions_enabled(true)
    .with_actions_log_dir(log_dir.clone());
    let app = router_with_state(state, build_cors("development", None).unwrap());

    let owner = db
        .create_user(
            "u-upd",
            "upd@example.com",
            "upd",
            Some("h"),
            "U",
            "",
            None,
            Role::User,
        )
        .await
        .unwrap();
    let repo = db
        .insert_repository("r-upd", &owner.id, "user", "upd", "private", "", "main")
        .await
        .unwrap();
    let doc = parse_workflow_yaml(
        br#"
name: CI
on: [push]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo hi
"#,
    )
    .unwrap();
    enqueue_run(
        &db,
        &repo.id,
        ".github/workflows/ci.yml",
        &doc,
        "push",
        "cafebabe",
        "refs/heads/main",
        Some(&owner.id),
    )
    .await
    .unwrap();

    let reg = mint_registration_token(&db).await.unwrap();
    let reg_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "r-upd",
                        "labels": ["ubuntu-latest"],
                        "token": reg
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let reg_v: serde_json::Value =
        serde_json::from_slice(&reg_res.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let runner_token = reg_v["runner_token"].as_str().unwrap();

    let fetch = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/fetch_task")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {runner_token}"))
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fetch.status(), StatusCode::OK);
    let fetch_v: serde_json::Value =
        serde_json::from_slice(&fetch.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let job_id = fetch_v["job_id"].as_str().expect("claimed job_id");
    let run_id = fetch_v["run_id"].as_str().expect("claimed run_id");
    // Runner needs repo coordinates to clone for checkout steps.
    assert_eq!(fetch_v["repository_owner"].as_str().unwrap(), "upd");
    assert_eq!(fetch_v["repository_name"].as_str().unwrap(), "upd");

    let start = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/update_task")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {runner_token}"))
                .body(Body::from(
                    serde_json::json!({ "job_id": job_id, "state": "in_progress" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(start.status(), StatusCode::OK);
    let job = db.find_action_job_by_id(job_id).await.unwrap().unwrap();
    assert_eq!(job.status, "in_progress");
    assert!(job.started_at.is_some(), "in_progress sets started_at");
    let run = db.find_action_run_by_id(run_id).await.unwrap().unwrap();
    assert_eq!(run.status, "in_progress", "run rolls up to in_progress");

    let log_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/update_log")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {runner_token}"))
                .body(Body::from(
                    serde_json::json!({
                        "run_id": run_id,
                        "job_id": job_id,
                        "chunk": "hello from runner\n"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(log_res.status(), StatusCode::OK);

    let upd = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/actions/update_task")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {runner_token}"))
                .body(Body::from(
                    serde_json::json!({
                        "job_id": job_id,
                        "state": "success"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upd.status(), StatusCode::OK);

    let job = db.find_action_job_by_id(job_id).await.unwrap().unwrap();
    assert_eq!(job.status, "success");
    assert!(
        job.finished_at.is_some(),
        "terminal status sets finished_at"
    );

    let run = db.find_action_run_by_id(run_id).await.unwrap().unwrap();
    assert_eq!(run.status, "success", "run rolls up to success");
    assert!(run.finished_at.is_some(), "terminal run sets finished_at");

    let runner = db
        .find_action_runner_by_token_hash(&octanest_api::actions::tokens::hash_token(runner_token))
        .await
        .unwrap()
        .unwrap();
    assert!(
        runner.last_online.is_some(),
        "protocol calls bump runner last_online"
    );

    let log_path = log_dir.join(run_id).join(format!("{job_id}.log"));
    let log_bytes = std::fs::read(&log_path).expect("log file written by update_log");
    assert!(
        String::from_utf8_lossy(&log_bytes).contains("hello from runner"),
        "update_log chunk must persist under ACTIONS_LOG_DIR"
    );

    let statuses = db.list_commit_statuses(&repo.id, "cafebabe").await.unwrap();
    let st = statuses
        .iter()
        .find(|s| s.context.contains("build"))
        .expect("commit status published for job");
    assert_eq!(st.state, "success");
}
