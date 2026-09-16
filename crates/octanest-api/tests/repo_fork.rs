//! SOC-04 Wave 0 stubs → green in 21-05.
//! Extends Phase 12 repo.fork: network id, public-only, one fork per owner+network,
//! head_valid_for_base helper (D-PR-01…03 / D-SOC-12…18).

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
        r#"{{"procedure":"repo.create","input":{{"name":"{name}","visibility":"{visibility}","description":"src","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}}}"#
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

async fn rpc_json(app: &axum::Router, cookie: &str, body: &str) -> serde_json::Value {
    let res = app
        .clone()
        .oneshot(rpc_req_with_cookie(body, cookie))
        .await
        .unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
#[ignore = "Wave 0 — turns green in 21-05"]
async fn repo_fork_public_ok_network_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("fork.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (src_c, src_v) = signup_and_login(&app, "src@ex.com", "srcown").await;
    verify_user(&db, src_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &src_c, "upstream", "public").await;

    let get = rpc_json(
        &app,
        &src_c,
        r#"{"procedure":"repo.get","input":{"owner":"srcown","name":"upstream"}}"#,
    )
    .await;
    assert_eq!(get["ok"], true, "{get}");
    let root_id = get["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(
        get["data"]["fork_network_id"].as_str().unwrap_or(""),
        root_id,
        "root fork_network_id == id"
    );

    let (fork_c, fork_v) = signup_and_login(&app, "fork@ex.com", "forkown").await;
    verify_user(&db, fork_v["data"]["id"].as_str().unwrap()).await;

    let forked = rpc_json(
        &app,
        &fork_c,
        r#"{"procedure":"repo.fork","input":{"owner":"srcown","name":"upstream"}}"#,
    )
    .await;
    assert_eq!(forked["ok"], true, "{forked}");
    assert_eq!(forked["data"]["is_fork"], true);
    assert_eq!(
        forked["data"]["fork_network_id"].as_str().unwrap(),
        root_id.as_str()
    );
    assert!(forked["data"]["forked_from"].is_object() || forked["data"]["forked_from_id"].is_string());
}

#[tokio::test]
#[ignore = "Wave 0 — turns green in 21-05"]
async fn repo_fork_private_source_denied() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("fork_priv.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (src_c, src_v) = signup_and_login(&app, "privsrc@ex.com", "privsrc").await;
    verify_user(&db, src_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &src_c, "secret", "private").await;

    let (fork_c, fork_v) = signup_and_login(&app, "try@ex.com", "tryfork").await;
    verify_user(&db, fork_v["data"]["id"].as_str().unwrap()).await;

    let forked = rpc_json(
        &app,
        &fork_c,
        r#"{"procedure":"repo.fork","input":{"owner":"privsrc","name":"secret"}}"#,
    )
    .await;
    assert_eq!(forked["ok"], false, "{forked}");
}

#[tokio::test]
#[ignore = "Wave 0 — turns green in 21-05"]
async fn repo_fork_one_per_owner_network() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("fork_dup.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (src_c, src_v) = signup_and_login(&app, "dupsrc@ex.com", "dupsrc").await;
    verify_user(&db, src_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &src_c, "net", "public").await;

    let (fork_c, fork_v) = signup_and_login(&app, "dupf@ex.com", "dupfork").await;
    verify_user(&db, fork_v["data"]["id"].as_str().unwrap()).await;

    let first = rpc_json(
        &app,
        &fork_c,
        r#"{"procedure":"repo.fork","input":{"owner":"dupsrc","name":"net"}}"#,
    )
    .await;
    assert_eq!(first["ok"], true, "{first}");

    let second = rpc_json(
        &app,
        &fork_c,
        r#"{"procedure":"repo.fork","input":{"owner":"dupsrc","name":"net","into_name":"net-copy"}}"#,
    )
    .await;
    assert_eq!(second["ok"], false, "second fork same network — {second}");
}

#[tokio::test]
#[ignore = "Wave 0 — turns green in 21-05 (exports fork_network::head_valid_for_base)"]
async fn repo_fork_head_valid_for_base() {
    // Placeholder until fork_network module ships in 21-05.
    // Expected: same repo → true; head.fork_network_id == base.id → true; else false.
    assert!(
        false,
        "head_valid_for_base helper not yet exported — implement in 21-05"
    );
}
