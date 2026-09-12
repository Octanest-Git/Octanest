//! GIT-01 Wave 0 stubs: verified create, unverified gate, duplicate name.
//!
//! RED until repo.create RPC + require_verified land (later Phase 07 plans).

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_db::Database;
use tower::ServiceExt;

async fn test_app(db: Database) -> axum::Router {
    let state = AppState::new(db, Arc::new(LogSink) as Arc<dyn EmailSender>, "development");
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
    let signup = app
        .clone()
        .oneshot(rpc_req(&signup_body))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let _ = signup.into_body().collect().await;

    let login_body = format!(
        r#"{{"procedure":"auth.login","input":{{"identifier":"{email}","password":"password1","remember_me":false}}}}"#
    );
    let login = app
        .clone()
        .oneshot(rpc_req(&login_body))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&login);
    let bytes = login.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (cookie, v)
}

/// Verified session can create a public repository (GIT-01 happy path).
#[tokio::test]
async fn repo_create_verified_happy_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("repo_create_ok.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "owner@ex.com", "owner1").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();

    // Mark verified so require_verified passes (D-11).
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let create = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"hello-world","visibility":"public","description":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(
        create.status(),
        StatusCode::OK,
        "Wave 0: repo.create must succeed for verified owner"
    );
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "Wave 0: repo.create ok=true — {v}");
    assert_eq!(
        v["data"]["name"], "hello-world",
        "Wave 0: created repo name"
    );
}

/// Unverified session must not create repos — auth.email_unverified (D-11 / T-07-01).
#[tokio::test]
async fn repo_create_unverified_email_unverified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("repo_create_unverified.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db).await;

    let (cookie, login_v) = signup_and_login(&app, "newbie@ex.com", "newbie1").await;
    assert_eq!(login_v["data"]["email_verified"], false);

    let create = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"blocked","visibility":"public","description":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(
        create.status(),
        StatusCode::FORBIDDEN,
        "Wave 0: unverified create must be forbidden"
    );
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        v["error"]["code"], "auth.email_unverified",
        "Wave 0: must surface auth.email_unverified — {v}"
    );
}

/// Second create of same owner+name among non-deleted → stable duplicate error (GIT-01).
#[tokio::test]
async fn repo_create_duplicate_name_stable_error() {
    assert!(
        false,
        "Wave 0: second repo.create same owner+name must return stable duplicate error (never overwrite)"
    );
}
