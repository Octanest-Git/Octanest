//! Phase 19 — runner protocol tracer (ACT-06 / D-ACT-07 / D-ACT-09 / D-ACT-18).

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::actions::dispatch::{dispatch_push_for_sha, enqueue_run};
use octanest_api::actions::parse_workflow_yaml;
use octanest_api::actions::mint_registration_token;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_core::Role;
use octanest_db::Database;
use octanest_git::{CliGitBackend, GitBackend};
use tower::ServiceExt;

async fn app_db() -> (axum::Router, Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::connect(&format!(
        "sqlite:{}",
        dir.path().join("r.db").display()
    ))
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
    std::env::set_var("OCTANEST_RUNNER_REGISTRATION_TOKEN", "bootstrap-secret-token");
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
