//! Usernames are immutable — `user.update_profile` must reject renames.

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oxidean_api::email::{EmailSender, LogSink};
use oxidean_api::{build_cors, router_with_state, AppState};
use oxidean_db::Database;
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
        .header("Oxidean-RPC-Version", "1")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn rpc_req_with_cookie(body: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/rpc")
        .header("content-type", "application/json")
        .header("Oxidean-RPC-Version", "1")
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

#[tokio::test]
async fn update_profile_rejects_username_change() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    tokio::fs::create_dir_all(&repos).await.unwrap();
    let url = format!("sqlite:{}", dir.path().join("immutable.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;

    let signup = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"renamer@ex.com","username":"alice","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&signup);
    let signup_bytes = signup.into_body().collect().await.unwrap().to_bytes();
    let signup_v: serde_json::Value = serde_json::from_slice(&signup_bytes).unwrap();
    let user_id = signup_v["data"]["id"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"hello","visibility":"public","description":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    assert!(repos.join("alice").join("hello.git").exists());

    let update = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"user.update_profile","input":{"display_name":"Alice","username":"bob","bio":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let update_bytes = update.into_body().collect().await.unwrap().to_bytes();
    let update_v: serde_json::Value = serde_json::from_slice(&update_bytes).unwrap();
    assert_eq!(update_v["ok"], false, "{update_v}");
    assert_eq!(update_v["error"]["code"], "auth.username_immutable");

    let me = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.me","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let me_bytes = me.into_body().collect().await.unwrap().to_bytes();
    let me_v: serde_json::Value = serde_json::from_slice(&me_bytes).unwrap();
    assert_eq!(me_v["data"]["username"], "alice");
    assert!(repos.join("alice").join("hello.git").exists());
    assert!(!repos.join("bob").exists());
}

#[tokio::test]
async fn update_profile_allows_same_username() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    tokio::fs::create_dir_all(&repos).await.unwrap();
    let url = format!("sqlite:{}", dir.path().join("same.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;

    let signup = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"same@ex.com","username":"alice","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&signup);

    let update = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"user.update_profile","input":{"display_name":"Alice Wonder","username":"alice","bio":"hi"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let update_bytes = update.into_body().collect().await.unwrap().to_bytes();
    let update_v: serde_json::Value = serde_json::from_slice(&update_bytes).unwrap();
    assert_eq!(update_v["ok"], true, "{update_v}");
    assert_eq!(update_v["data"]["username"], "alice");
    assert_eq!(update_v["data"]["display_name"], "Alice Wonder");
    assert_eq!(update_v["data"]["bio"], "hi");
}
