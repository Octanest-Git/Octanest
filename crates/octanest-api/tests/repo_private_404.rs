//! GIT-05 / D-25 Wave 0 stubs: private non-owner and missing → identical not_found.
//!
//! RED until browse ACL helpers land (T-07-02).

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

/// Missing repo and private non-owner must share `repo.not_found` (no existence leak).
#[tokio::test]
async fn repo_private_404_identical_not_found_for_missing_and_private() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("repo_private_404.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db).await;

    // Establish a signed-in stranger session (verified or not — ACL is ownership).
    let signup = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"stranger@ex.com","username":"stranger1","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&signup);

    let missing = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.get","input":{"owner":"nobody","name":"missing-repo"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let missing_bytes = missing.into_body().collect().await.unwrap().to_bytes();
    let missing_v: serde_json::Value = serde_json::from_slice(&missing_bytes).unwrap();

    let private = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.get","input":{"owner":"owner1","name":"secret-private"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let private_bytes = private.into_body().collect().await.unwrap().to_bytes();
    let private_v: serde_json::Value = serde_json::from_slice(&private_bytes).unwrap();

    assert_eq!(
        missing_v["error"]["code"], "repo.not_found",
        "Wave 0: missing repo → repo.not_found — {missing_v}"
    );
    assert_eq!(
        private_v["error"]["code"], "repo.not_found",
        "Wave 0: private non-owner → repo.not_found (identical) — {private_v}"
    );
    assert_eq!(
        missing_v["error"]["code"], private_v["error"]["code"],
        "Wave 0: codes must match to avoid existence leak"
    );
}
