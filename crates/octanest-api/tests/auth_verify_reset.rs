//! AUTH-04: verify issue/resend/rate-limit + magic/OTP consume.

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::auth::verify_reset;
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

async fn test_app(db: Database) -> axum::Router {
    let state = AppState::new(db, Arc::new(LogSink) as Arc<dyn EmailSender>, "development");
    let cors = build_cors("development", None).expect("cors");
    router_with_state(state, cors)
}

async fn test_app_with_recorder(db: Database) -> (axum::Router, Arc<RecordingSender>) {
    let recorder = Arc::new(RecordingSender::default());
    let state = AppState::new(
        db,
        recorder.clone() as Arc<dyn EmailSender>,
        "development",
    );
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

async fn signup_user(
    app: &axum::Router,
    email: &str,
    username: &str,
) -> (String, String, serde_json::Value) {
    let body = format!(
        r#"{{"procedure":"auth.signup","input":{{"email":"{email}","username":"{username}","password":"password1"}}}}"#
    );
    let signup = app
        .clone()
        .oneshot(rpc_req(&body))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&signup);
    let bytes = signup.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let user_id = v["data"]["id"].as_str().expect("id").to_string();
    (cookie, user_id, v)
}

/// Backdate token `created_at` so the 60s min-interval does not block the next issue.
async fn backdate_verify_created_at(db: &Database, user_id: &str, secs_ago: i64) {
    let at = (chrono::Utc::now() - chrono::Duration::seconds(secs_ago))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    db.set_email_token_created_at(user_id, "verify", &at)
        .await
        .expect("backdate");
}

fn extract_otp_from_verify_email(text: &str) -> String {
    let marker = "Or enter this 8-digit code:";
    let after = text
        .split_once(marker)
        .unwrap_or_else(|| panic!("missing otp phrase in: {text}"))
        .1;
    // OTP is the first 8-digit run after the marker (ignore later "30 minutes").
    let mut code = String::new();
    for c in after.chars() {
        if c.is_ascii_digit() {
            code.push(c);
            if code.len() == 8 {
                return code;
            }
        } else if !code.is_empty() {
            code.clear();
        }
    }
    panic!("no 8-digit otp after marker in: {text}");
}

#[tokio::test]
async fn issue_otp_consume_sets_email_verified_on_me() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("verify_reset.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let app = test_app(db.clone()).await;

    let (cookie, user_id, signup_v) = signup_user(&app, "user@ex.com", "user1").await;
    assert_eq!(signup_v["data"]["email_verified"], false);

    let me_before = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.me","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(me_before.status(), StatusCode::OK);
    let me_before_bytes = me_before.into_body().collect().await.unwrap().to_bytes();
    let me_before_v: serde_json::Value = serde_json::from_slice(&me_before_bytes).unwrap();
    assert_eq!(me_before_v["data"]["email_verified"], false);

    let secrets = verify_reset::issue_verify(&db, &user_id)
        .await
        .expect("issue_verify");
    assert_eq!(secrets.otp.len(), 8);
    assert!(secrets.otp.chars().all(|c| c.is_ascii_digit()));

    let verify_body = format!(
        r#"{{"procedure":"auth.verify","input":{{"code":"{}"}}}}"#,
        secrets.otp
    );
    let verify = app
        .clone()
        .oneshot(rpc_req_with_cookie(&verify_body, &cookie))
        .await
        .unwrap();
    assert_eq!(verify.status(), StatusCode::OK);
    let verify_bytes = verify.into_body().collect().await.unwrap().to_bytes();
    let verify_v: serde_json::Value = serde_json::from_slice(&verify_bytes).unwrap();
    assert_eq!(verify_v["data"]["email_verified"], true);

    let me_after = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.me","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(me_after.status(), StatusCode::OK);
    let me_after_bytes = me_after.into_body().collect().await.unwrap().to_bytes();
    let me_after_v: serde_json::Value = serde_json::from_slice(&me_after_bytes).unwrap();
    assert_eq!(me_after_v["data"]["email_verified"], true);
}

#[tokio::test]
async fn request_verify_sends_magic_and_otp_email() {
    std::env::set_var("OCTANEST_PUBLIC_ORIGIN", "https://app.example.com");
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("req_verify.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let (app, recorder) = test_app_with_recorder(db).await;

    let (cookie, _, _) = signup_user(&app, "v@ex.com", "verifyme").await;
    // Clear welcome (and any signup auto-verify) so we assert request_verify alone.
    recorder.sent.lock().expect("lock").clear();

    let res = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.request_verify","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true);

    let sent = recorder.sent.lock().expect("lock");
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].subject, "Verify your Octanest email");
    assert!(
        sent[0].text.contains("https://app.example.com/verify?token="),
        "body: {}",
        sent[0].text
    );
    assert!(
        sent[0].text.contains("Or enter this 8-digit code:"),
        "body: {}",
        sent[0].text
    );
    assert!(
        sent[0].text.contains("30 minutes") || sent[0].text.contains("30 minute"),
        "body: {}",
        sent[0].text
    );
    std::env::remove_var("OCTANEST_PUBLIC_ORIGIN");
}

#[tokio::test]
async fn magic_token_consume_sets_verified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("magic.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let app = test_app(db.clone()).await;

    let (cookie, user_id, _) = signup_user(&app, "magic@ex.com", "magicuser").await;
    let secrets = verify_reset::issue_verify(&db, &user_id)
        .await
        .expect("issue");

    let verify_body = format!(
        r#"{{"procedure":"auth.verify","input":{{"token":"{}"}}}}"#,
        secrets.magic
    );
    let verify = app
        .oneshot(rpc_req_with_cookie(&verify_body, &cookie))
        .await
        .unwrap();
    assert_eq!(verify.status(), StatusCode::OK);
    let bytes = verify.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["data"]["email_verified"], true);
}

#[tokio::test]
async fn verify_wrong_session_user_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("wrong_user.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let app = test_app(db.clone()).await;

    let (_c1, user1, _) = signup_user(&app, "a@ex.com", "usera").await;
    let (cookie2, _, _) = signup_user(&app, "b@ex.com", "userb").await;
    let secrets = verify_reset::issue_verify(&db, &user1)
        .await
        .expect("issue");

    let verify_body = format!(
        r#"{{"procedure":"auth.verify","input":{{"code":"{}"}}}}"#,
        secrets.otp
    );
    let verify = app
        .oneshot(rpc_req_with_cookie(&verify_body, &cookie2))
        .await
        .unwrap();
    assert_eq!(verify.status(), StatusCode::BAD_REQUEST);
    let bytes = verify.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "auth.invalid_token");
}

#[tokio::test]
async fn resend_replaces_prior_and_rate_limits_within_60s() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("resend.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let (app, recorder) = test_app_with_recorder(db.clone()).await;

    let (cookie, user_id, _) = signup_user(&app, "r@ex.com", "resender").await;
    recorder.sent.lock().expect("lock").clear();

    let first = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.resend_verify","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let first_otp = {
        let sent = recorder.sent.lock().expect("lock");
        assert_eq!(sent.len(), 1);
        extract_otp_from_verify_email(&sent[0].text)
    };

    let second = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.resend_verify","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::BAD_REQUEST);
    let bytes = second.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "auth.rate_limited");

    // Old OTP must be invalid after a successful replace — issue again after backdate.
    backdate_verify_created_at(&db, &user_id, 61).await;
    recorder.sent.lock().expect("lock").clear();
    let third = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.resend_verify","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(third.status(), StatusCode::OK);

    let old_verify = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            &format!(r#"{{"procedure":"auth.verify","input":{{"code":"{first_otp}"}}}}"#),
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(old_verify.status(), StatusCode::BAD_REQUEST);

    let new_otp = {
        let sent = recorder.sent.lock().expect("lock");
        assert_eq!(sent.len(), 1);
        extract_otp_from_verify_email(&sent[0].text)
    };
    assert_ne!(first_otp, new_otp);
}

#[tokio::test]
async fn sixth_issue_within_hour_rate_limited() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("hourly.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let app = test_app(db.clone()).await;

    let (cookie, user_id, _) = signup_user(&app, "h@ex.com", "hourly").await;

    for i in 0..5 {
        if i > 0 {
            backdate_verify_created_at(&db, &user_id, 61).await;
        }
        let res = app
            .clone()
            .oneshot(rpc_req_with_cookie(
                r#"{"procedure":"auth.request_verify","input":{}}"#,
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK, "issue {i} should succeed");
    }

    backdate_verify_created_at(&db, &user_id, 61).await;
    let sixth = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"auth.request_verify","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(sixth.status(), StatusCode::BAD_REQUEST);
    let bytes = sixth.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "auth.rate_limited");
}

#[tokio::test]
async fn ten_failed_otp_attempts_invalidate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("attempts.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let app = test_app(db.clone()).await;

    let (cookie, user_id, _) = signup_user(&app, "t@ex.com", "tryhard").await;
    let secrets = verify_reset::issue_verify(&db, &user_id)
        .await
        .expect("issue");

    for i in 0..10 {
        let res = app
            .clone()
            .oneshot(rpc_req_with_cookie(
                r#"{"procedure":"auth.verify","input":{"code":"00000000"}}"#,
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "attempt {i}");
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"]["code"], "auth.invalid_token");
    }

    let good = app
        .oneshot(rpc_req_with_cookie(
            &format!(
                r#"{{"procedure":"auth.verify","input":{{"code":"{}"}}}}"#,
                secrets.otp
            ),
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(good.status(), StatusCode::BAD_REQUEST);
    let bytes = good.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "auth.invalid_token");
}
