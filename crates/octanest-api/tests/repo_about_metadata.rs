//! Issue #23 — repo.updateMetadata, watch/unwatch, fork_count bump.

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_db::Database;
use tower::ServiceExt;

async fn test_app(db: Database, repos_dir: std::path::PathBuf) -> axum::Router {
    let state = AppState::new(db, Arc::new(LogSink) as Arc<dyn EmailSender>, "development")
        .with_repos_dir(repos_dir);
    let cors = build_cors("development", None).expect("cors");
    router_with_state(state, cors)
}

fn rpc_req(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/rpc")
        .header("content-type", "application/json")
        .header("Octanest-RPC-Version", "1")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn rpc_req_with_cookie(body: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/rpc")
        .header("content-type", "application/json")
        .header("Octanest-RPC-Version", "1")
        .header("cookie", cookie)
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn session_cookie_from_response(res: &axum::http::Response<Body>) -> String {
    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie")
        .to_str()
        .unwrap();
    set_cookie.split(';').next().unwrap().trim().to_string()
}

async fn signup_and_login(
    app: &axum::Router,
    email: &str,
    username: &str,
) -> (String, serde_json::Value) {
    let signup_body = format!(
        r#"{{"procedure":"auth.signup","input":{{"email":"{email}","username":"{username}","password":"password1"}}}}"#
    );
    let signup = app.clone().oneshot(rpc_req(&signup_body)).await.unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let _ = signup.into_body().collect().await;

    let login_body = format!(
        r#"{{"procedure":"auth.login","input":{{"identifier":"{email}","password":"password1","remember_me":false}}}}"#
    );
    let login = app.clone().oneshot(rpc_req(&login_body)).await.unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&login);
    let bytes = login.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (cookie, v)
}

async fn verify_user(db: &Database, user_id: &str) {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(user_id, &now)
        .await
        .expect("verify");
}

async fn create_repo(app: &axum::Router, cookie: &str, name: &str, visibility: &str) {
    let body = format!(
        r#"{{"procedure":"repo.create","input":{{"name":"{name}","visibility":"{visibility}","description":"","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}}}"#
    );
    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(&body, cookie))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "repo.create — {v}");
}

async fn rpc_json(app: &axum::Router, cookie: Option<&str>, body: &str) -> serde_json::Value {
    let res = match cookie {
        Some(c) => app.clone().oneshot(rpc_req_with_cookie(body, c)).await.unwrap(),
        None => app.clone().oneshot(rpc_req(body)).await.unwrap(),
    };
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn repo_about_admin_updates_metadata() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("about_meta.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, v) = signup_and_login(&app, "meta@ex.com", "metaown").await;
    verify_user(&db, v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &cookie, "hello", "public").await;

    let updated = rpc_json(
        &app,
        Some(&cookie),
        r#"{"procedure":"repo.updateMetadata","input":{"owner":"metaown","name":"hello","description":"A cool repo","homepage":"https://example.com","topics":["rust","cli"]}}"#,
    )
    .await;
    assert_eq!(updated["ok"], true, "{updated}");
    assert_eq!(updated["data"]["description"], "A cool repo");
    assert_eq!(updated["data"]["homepage"], "https://example.com");
    let topics = updated["data"]["topics"].as_array().unwrap();
    assert_eq!(topics.len(), 2);
    assert!(topics.iter().any(|t| t == "cli"));
    assert!(topics.iter().any(|t| t == "rust"));
}

#[tokio::test]
async fn repo_about_non_admin_soft_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("about_deny.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (owner_c, owner_v) = signup_and_login(&app, "own@ex.com", "ownuser").await;
    verify_user(&db, owner_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &owner_c, "pub", "public").await;

    let (other_c, other_v) = signup_and_login(&app, "other@ex.com", "otheru").await;
    verify_user(&db, other_v["data"]["id"].as_str().unwrap()).await;

    let denied = rpc_json(
        &app,
        Some(&other_c),
        r#"{"procedure":"repo.updateMetadata","input":{"owner":"ownuser","name":"pub","description":"Nope"}}"#,
    )
    .await;
    assert_eq!(denied["ok"], false, "{denied}");
    let err = denied["error"]["code"].as_str().unwrap_or("");
    assert!(
        err.contains("not_found") || err == "repo.not_found",
        "expected not_found, got {denied}"
    );
}

#[tokio::test]
async fn repo_about_watch_unwatch_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("about_watch.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, v) = signup_and_login(&app, "watch@ex.com", "watchown").await;
    verify_user(&db, v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &cookie, "hello", "public").await;

    let watch = rpc_json(
        &app,
        Some(&cookie),
        r#"{"procedure":"repo.watch","input":{"owner":"watchown","name":"hello"}}"#,
    )
    .await;
    assert_eq!(watch["ok"], true, "{watch}");
    assert_eq!(watch["data"]["watch_count"], 1);
    assert_eq!(watch["data"]["viewer_is_watching"], true);

    let watch2 = rpc_json(
        &app,
        Some(&cookie),
        r#"{"procedure":"repo.watch","input":{"owner":"watchown","name":"hello"}}"#,
    )
    .await;
    assert_eq!(watch2["ok"], true, "{watch2}");
    assert_eq!(watch2["data"]["watch_count"], 1);

    let unwatch = rpc_json(
        &app,
        Some(&cookie),
        r#"{"procedure":"repo.unwatch","input":{"owner":"watchown","name":"hello"}}"#,
    )
    .await;
    assert_eq!(unwatch["ok"], true, "{unwatch}");
    assert_eq!(unwatch["data"]["watch_count"], 0);
    assert_eq!(unwatch["data"]["viewer_is_watching"], false);
}

#[tokio::test]
async fn repo_about_fork_bumps_fork_count() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("about_fork.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (owner_c, owner_v) = signup_and_login(&app, "src@ex.com", "srcown").await;
    verify_user(&db, owner_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &owner_c, "upstream", "public").await;

    let (forker_c, forker_v) = signup_and_login(&app, "fork@ex.com", "forku").await;
    verify_user(&db, forker_v["data"]["id"].as_str().unwrap()).await;

    let forked = rpc_json(
        &app,
        Some(&forker_c),
        r#"{"procedure":"repo.fork","input":{"owner":"srcown","name":"upstream"}}"#,
    )
    .await;
    assert_eq!(forked["ok"], true, "{forked}");
    assert_eq!(forked["data"]["fork_count"], 1);
    assert_eq!(forked["data"]["is_fork"], true);

    let source = rpc_json(
        &app,
        Some(&owner_c),
        r#"{"procedure":"repo.get","input":{"owner":"srcown","name":"upstream"}}"#,
    )
    .await;
    assert_eq!(source["ok"], true, "{source}");
    assert_eq!(source["data"]["fork_count"], 1);
}
