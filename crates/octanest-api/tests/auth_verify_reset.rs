//! AUTH-04 tracer: verify OTP → email_verified on auth.me.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::auth::verify_reset;
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

#[tokio::test]
async fn issue_otp_consume_sets_email_verified_on_me() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("verify_reset.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let app = test_app(db.clone()).await;

    let signup = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"user@ex.com","username":"user1","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&signup);
    let signup_bytes = signup.into_body().collect().await.unwrap().to_bytes();
    let signup_v: serde_json::Value = serde_json::from_slice(&signup_bytes).unwrap();
    assert_eq!(signup_v["data"]["email_verified"], false);
    let user_id = signup_v["data"]["id"].as_str().expect("id").to_string();

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
