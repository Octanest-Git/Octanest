//! Username change must move `{repos_dir}/{old}/` → `{new}/` before DB commit.

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

#[tokio::test]
async fn update_profile_renames_owner_repos_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    tokio::fs::create_dir_all(&repos).await.unwrap();
    let url = format!("sqlite:{}", dir.path().join("migrate.db").display());
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
    let create_bytes = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_bytes).unwrap();
    assert_eq!(create_v["ok"], true);
    assert!(repos.join("alice").join("hello.git").exists());

    let update = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"user.update_profile","input":{"display_name":"Alice","username":"bob","bio":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);
    let update_bytes = update.into_body().collect().await.unwrap().to_bytes();
    let update_v: serde_json::Value = serde_json::from_slice(&update_bytes).unwrap();
    assert_eq!(update_v["ok"], true);
    assert_eq!(update_v["data"]["username"], "bob");

    assert!(!repos.join("alice").exists());
    assert!(repos.join("bob").join("hello.git").exists());

    let got = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.get","input":{"owner":"bob","name":"hello"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(got.status(), StatusCode::OK);
    let got_bytes = got.into_body().collect().await.unwrap().to_bytes();
    let got_v: serde_json::Value = serde_json::from_slice(&got_bytes).unwrap();
    assert_eq!(got_v["ok"], true);
    assert_eq!(got_v["data"]["owner_username"], "bob");
    assert_eq!(got_v["data"]["name"], "hello");
}

#[tokio::test]
async fn update_profile_username_conflict_when_dest_dir_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    tokio::fs::create_dir_all(repos.join("alice").join("hello.git"))
        .await
        .unwrap();
    tokio::fs::create_dir_all(repos.join("taken"))
        .await
        .unwrap();
    let url = format!("sqlite:{}", dir.path().join("conflict.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;

    let signup = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"c@ex.com","username":"alice","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&signup);
    let signup_bytes = signup.into_body().collect().await.unwrap().to_bytes();
    let signup_v: serde_json::Value = serde_json::from_slice(&signup_bytes).unwrap();
    let user_id = signup_v["data"]["id"].as_str().unwrap().to_string();
    // Seed DB repo row is optional for conflict — FS dest already exists.
    let _ = user_id;

    let update = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"user.update_profile","input":{"display_name":"Alice","username":"taken","bio":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let update_status = update.status();
    let update_bytes = update.into_body().collect().await.unwrap().to_bytes();
    let update_v: serde_json::Value = serde_json::from_slice(&update_bytes).unwrap();
    assert!(
        update_status.is_success() || update_status.as_u16() == 400,
        "unexpected status {update_status}: {update_v}"
    );
    assert_eq!(update_v["ok"], false, "{update_v}");
    assert_eq!(update_v["error"]["code"], "repo.owner_dir_conflict");

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
    assert!(repos.join("alice").exists());
}
