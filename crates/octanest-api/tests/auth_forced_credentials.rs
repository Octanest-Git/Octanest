//! Wave 0 (06-00): AUTH-06 forced credential confirm stubs (D-16/D-17).
//! Intentionally RED until 06-02 implements `auth.confirm_admin_credentials`.

mod support;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_db::Database;
use std::sync::Arc;
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

fn rpc_req_cookie(body: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/rpc")
        .header("content-type", "application/json")
        .header("Octanest-RPC-Version", "1")
        .header("cookie", cookie)
        .body(Body::from(body.to_owned()))
        .unwrap()
}

async fn seed_env_admin(db: &Database) {
    std::env::set_var("OCTANEST_ADMIN_EMAIL", "admin@example.com");
    std::env::set_var("OCTANEST_ADMIN_PASSWORD", "adminpass1");
    std::env::remove_var("OCTANEST_ALLOW_SIGNUP");
    octanest_api::auth::seed::maybe_seed_admin(db)
        .await
        .expect("seed");
    std::env::remove_var("OCTANEST_ADMIN_EMAIL");
    std::env::remove_var("OCTANEST_ADMIN_PASSWORD");
}

/// Try login; return Set-Cookie value when present (Wave 0 may lack session until seed username lands).
async fn try_admin_cookie(app: axum::Router) -> Option<String> {
    let res = app
        .oneshot(rpc_req(
            r#"{"procedure":"auth.login","input":{"identifier":"admin@example.com","password":"adminpass1"}}"#,
        ))
        .await
        .unwrap();
    res.headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

/// D-16: confirm rejects username equal to `system-administrator` (case-insensitive).
#[tokio::test]
async fn confirm_admin_rejects_default_system_administrator_username() {
    let _env = support::lock_admin_env();
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("forced_reject.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    seed_env_admin(&db).await;

    let app = test_app(db.clone()).await;
    let cookie = try_admin_cookie(app).await.unwrap_or_default();
    let app2 = test_app(db).await;
    let req = if cookie.is_empty() {
        rpc_req(
            r#"{"procedure":"auth.confirm_admin_credentials","input":{"username":"system-administrator","keep_password":true}}"#,
        )
    } else {
        rpc_req_cookie(
            r#"{"procedure":"auth.confirm_admin_credentials","input":{"username":"system-administrator","keep_password":true}}"#,
            &cookie,
        )
    };
    let res = app2.oneshot(req).await.unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["error"]["code"], "auth.invalid_username",
        "forced confirm must reject default username system-administrator"
    );
}

/// D-17: keep-password path succeeds when username changes away from default.
#[tokio::test]
async fn confirm_admin_keep_password_ok_with_new_username() {
    let _env = support::lock_admin_env();
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("forced_keep.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    seed_env_admin(&db).await;

    let app = test_app(db.clone()).await;
    let cookie = try_admin_cookie(app).await.unwrap_or_default();
    let app2 = test_app(db.clone()).await;
    let req = if cookie.is_empty() {
        rpc_req(
            r#"{"procedure":"auth.confirm_admin_credentials","input":{"username":"forge-admin","keep_password":true}}"#,
        )
    } else {
        rpc_req_cookie(
            r#"{"procedure":"auth.confirm_admin_credentials","input":{"username":"forge-admin","keep_password":true}}"#,
            &cookie,
        )
    };
    let res = app2.oneshot(req).await.unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        v["ok"], true,
        "keep_password=true with non-default username must succeed"
    );
    assert_eq!(v["data"]["username"], "forge-admin");

    let user = db
        .find_user_by_email("admin@example.com")
        .await
        .expect("find")
        .expect("user");
    // Wave 0 RED until 06-01/06-02: must_change_credentials cleared on confirm.
    let must_change = true;
    let _ = user;
    assert!(
        !must_change,
        "successful confirm must clear must_change_credentials"
    );
}
