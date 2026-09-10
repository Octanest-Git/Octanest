//! AUTH-01: local signup with Set-Cookie, uniqueness, reserved username, welcome email.

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::email::{EmailError, EmailSender, LogSink, OutboundEmail};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_db::Database;
use tower::ServiceExt;

#[derive(Default)]
struct RecordingSender {
    sent: Mutex<Vec<OutboundEmail>>,
}

#[async_trait::async_trait]
impl EmailSender for RecordingSender {
    async fn send(&self, msg: OutboundEmail) -> Result<(), EmailError> {
        self.sent.lock().expect("lock").push(msg);
        Ok(())
    }
}

async fn app_with_recorder(db: Database) -> (axum::Router, Arc<RecordingSender>) {
    let recorder = Arc::new(RecordingSender::default());
    let state = AppState::new(db, recorder.clone() as Arc<dyn EmailSender>, "development");
    let cors = build_cors("development", None).expect("cors");
    (router_with_state(state, cors), recorder)
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

#[tokio::test]
async fn signup_sets_cookie_and_sends_welcome() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("signup.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let (app, recorder) = app_with_recorder(db).await;

    let res = app
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"Ada@Example.com","username":"ada","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie")
        .to_str()
        .unwrap();
    assert!(
        set_cookie.contains("octanest_session="),
        "cookie: {set_cookie}"
    );
    assert!(set_cookie.contains("HttpOnly") || set_cookie.to_ascii_lowercase().contains("httponly"));

    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["email"], "ada@example.com");
    assert_eq!(v["data"]["username"], "ada");

    let sent = recorder.sent.lock().expect("lock");
    assert_eq!(sent.len(), 2, "welcome + verify emails");
    assert!(
        sent.iter().any(|m| m.subject == "Welcome to Octanest"),
        "missing welcome"
    );
    let verify = sent
        .iter()
        .find(|m| m.subject == "Verify your Octanest email")
        .expect("verify email");
    assert!(verify.text.contains("Or enter this 8-digit code:"));
    assert!(verify.text.contains("/verify?token="));
    assert_eq!(verify.to, "ada@example.com");
}

#[tokio::test]
async fn signup_duplicate_returns_taken() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("dup.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let (app, _) = app_with_recorder(db).await;

    let res1 = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"a@ex.com","username":"alice","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    let res2 = app
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"a@ex.com","username":"alice2","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::BAD_REQUEST);
    let bytes = res2.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "auth.taken");
}

#[tokio::test]
async fn signup_reserved_username() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("reserved.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let (app, _) = app_with_recorder(db).await;

    let res = app
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"x@ex.com","username":"admin","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "auth.reserved_username");
}

#[tokio::test]
async fn signup_works_with_log_sink_default_router() {
    // Smoke: default router path (LogSink) still accepts signup.
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("logsink.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let state = AppState::new(db, Arc::new(LogSink) as Arc<dyn EmailSender>, "development");
    let cors = build_cors("development", None).expect("cors");
    let app = router_with_state(state, cors);

    let res = app
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"b@ex.com","username":"bob","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn signup_open_without_invite_fields_auth05() {
    // AUTH-05 / D-08: signup succeeds with only email/username/password — no invite schema.
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("open.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let (app, _) = app_with_recorder(db).await;

    let body = r#"{"procedure":"auth.signup","input":{"email":"open@ex.com","username":"opener","password":"password1"}}"#;
    assert!(
        !body.contains("invite"),
        "signup request must not carry invite fields"
    );
    let res = app.oneshot(rpc_req(body)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["email_verified"], false);
}

#[tokio::test]
async fn seeded_admin_is_auto_verified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("admin_seed.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

    std::env::set_var("OCTANEST_ADMIN_EMAIL", "Admin@Example.com");
    std::env::set_var("OCTANEST_ADMIN_PASSWORD", "adminpass1");
    octanest_api::auth::seed::maybe_seed_admin(&db)
        .await
        .expect("seed");
    std::env::remove_var("OCTANEST_ADMIN_EMAIL");
    std::env::remove_var("OCTANEST_ADMIN_PASSWORD");

    let user = db
        .find_user_by_email("admin@example.com")
        .await
        .expect("find")
        .expect("seeded user");
    assert!(user.is_admin);
    assert!(
        user.email_verified_at.is_some(),
        "D-04: seeded admin must be auto-verified"
    );
    // Second seed is a no-op when users exist.
    std::env::set_var("OCTANEST_ADMIN_EMAIL", "other@example.com");
    std::env::set_var("OCTANEST_ADMIN_PASSWORD", "adminpass1");
    octanest_api::auth::seed::maybe_seed_admin(&db)
        .await
        .expect("second seed");
    std::env::remove_var("OCTANEST_ADMIN_EMAIL");
    std::env::remove_var("OCTANEST_ADMIN_PASSWORD");
    assert_eq!(db.count_users().await.expect("count"), 1);
}
