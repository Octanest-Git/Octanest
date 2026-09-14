//! ISS-01 create/list/get + per-repo #N (D-ISS-01 / D-ISS-20).
//!
//! Edit/close/reopen/history remain Wave 0 RED until 11-04.

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
        r#"{{"procedure":"repo.create","input":{{"name":"{name}","visibility":"{visibility}","description":""}}}}"#
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

/// Verified Write+ can create an issue and receives per-repo `#1` (ISS-01 / D-ISS-01).
#[tokio::test]
async fn issue_lifecycle_create_allocates_per_repo_number() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("issue_create_n1.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "issowner@ex.com", "issowner").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;
    create_repo(&app, &cookie, "hello", "public").await;

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"issowner","name":"hello","title":"First","body":"body"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "issue.create ok — {v}");
    assert_eq!(v["data"]["number"], 1, "first issue is #1 — {v}");
    assert_eq!(v["data"]["title"], "First");
    assert_eq!(v["data"]["state"], "open");
    assert_eq!(v["data"]["author_username"], "issowner");

    let get = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.get","input":{"owner":"issowner","name":"hello","number":1}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let get_bytes = get.into_body().collect().await.unwrap().to_bytes();
    let get_v: serde_json::Value = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(get_v["ok"], true, "issue.get — {get_v}");
    assert_eq!(get_v["data"]["number"], 1);

    let list = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.list","input":{"owner":"issowner","name":"hello"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let list_bytes = list.into_body().collect().await.unwrap().to_bytes();
    let list_v: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    assert_eq!(list_v["ok"], true, "issue.list — {list_v}");
    assert_eq!(list_v["data"]["total"], 1);
    assert_eq!(list_v["data"]["issues"][0]["number"], 1);
}

/// Second create in same repo gets `#2`; second repo starts at `#1` (D-ISS-01).
#[tokio::test]
async fn issue_lifecycle_second_create_monotonic_number() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("issue_create_n2.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "mono@ex.com", "monoown").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;
    create_repo(&app, &cookie, "repo-a", "public").await;
    create_repo(&app, &cookie, "repo-b", "public").await;

    let c1 = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"monoown","name":"repo-a","title":"A1"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let c1b = c1.into_body().collect().await.unwrap().to_bytes();
    let c1v: serde_json::Value = serde_json::from_slice(&c1b).unwrap();
    assert_eq!(c1v["ok"], true, "{c1v}");
    assert_eq!(c1v["data"]["number"], 1);

    let c2 = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"monoown","name":"repo-a","title":"A2"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let c2b = c2.into_body().collect().await.unwrap().to_bytes();
    let c2v: serde_json::Value = serde_json::from_slice(&c2b).unwrap();
    assert_eq!(c2v["ok"], true, "{c2v}");
    assert_eq!(c2v["data"]["number"], 2, "second in same repo is #2");

    let b1 = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"monoown","name":"repo-b","title":"B1"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let b1b = b1.into_body().collect().await.unwrap().to_bytes();
    let b1v: serde_json::Value = serde_json::from_slice(&b1b).unwrap();
    assert_eq!(b1v["ok"], true, "{b1v}");
    assert_eq!(b1v["data"]["number"], 1, "second repo starts at #1");
}

/// Author or Write+ may edit title/body after create (ISS-01 / D-ISS-03).
#[tokio::test]
async fn issue_lifecycle_edit_title_body() {
    assert!(
        false,
        "Wave 0: issue.edit must allow author + Write+ to change title/body (ISS-01 / D-ISS-03)"
    );
}

/// Lifecycle is open ↔ closed; reopen allowed (ISS-01 / D-ISS-02).
#[tokio::test]
async fn issue_lifecycle_close_and_reopen() {
    assert!(
        false,
        "Wave 0: issue.close / issue.reopen must toggle open↔closed (ISS-01 / D-ISS-02)"
    );
}

/// Full edit history trail for title/body (ISS-01 / D-ISS-04).
#[tokio::test]
async fn issue_history_full_title_body_trail() {
    assert!(
        false,
        "Wave 0: issue.history must return full title/body revision trail (ISS-01 / D-ISS-04)"
    );
}
