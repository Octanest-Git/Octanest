//! PR-07 merge strategy settings.

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
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
    res.headers()
        .get("set-cookie")
        .expect("Set-Cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .trim()
        .to_string()
}

#[tokio::test]
async fn pull_merge_settings_defaults_all_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("ms_def.db").display());
    let db = Database::connect(&url).await.unwrap();
    db.migrate().await.unwrap();
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let signup = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"d@ex.com","username":"down","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    let _ = signup.into_body().collect().await;
    let login = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.login","input":{"identifier":"d@ex.com","password":"password1","remember_me":false}}"#,
        ))
        .await
        .unwrap();
    let cookie = session_cookie_from_response(&login);
    let bytes = login.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let uid = v["data"]["id"].as_str().unwrap();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(uid, &now).await.unwrap();

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"r","visibility":"public","description":"","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let cbytes = create.into_body().collect().await.unwrap().to_bytes();
    let cv: serde_json::Value = serde_json::from_slice(&cbytes).unwrap();
    assert_eq!(cv["ok"], true, "{cv}");

    let get = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.mergeSettings.get","input":{"owner":"down","name":"r"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let gbytes = get.into_body().collect().await.unwrap().to_bytes();
    let gv: serde_json::Value = serde_json::from_slice(&gbytes).unwrap();
    assert_eq!(gv["ok"], true, "{gv}");
    assert_eq!(gv["data"]["allow_merge_commit"], true);
    assert_eq!(gv["data"]["allow_squash_merge"], true);
    assert_eq!(gv["data"]["allow_rebase_merge"], true);
}
