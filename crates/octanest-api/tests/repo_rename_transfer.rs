//! Wave 0 / Phase 15: GIT-16/17 rename, transfer, redirect stubs (D-REL-07..11).
//! Intentionally ignored until plans 15-03 / 15-04 turn them green.

#![allow(dead_code)]

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

/// GIT-16 / D-REL-07 / D-REL-08: Admin rename moves disk+DB and inserts redirect.
#[tokio::test]
#[ignore = "Wave 0: green in 15-03"]
async fn repo_rename_admin_moves_disk_and_inserts_redirect() {
    let _ = (test_app, signup_and_login);
    assert!(
        false,
        "Wave 0 stub: Admin rename moves bare dir + DB name and inserts repository_redirects (GIT-16, D-REL-07, D-REL-08)"
    );
}

/// D-REL-07: non-admin rename → soft repo.not_found.
#[tokio::test]
#[ignore = "Wave 0: green in 15-03"]
async fn repo_rename_non_admin_soft_not_found() {
    assert!(
        false,
        "Wave 0 stub: non-Admin rename returns identical soft repo.not_found (D-REL-07, T-15-01)"
    );
}

/// D-REL-08: redirect resolve on old path within retention.
#[tokio::test]
#[ignore = "Wave 0: green in 15-03"]
async fn redirect_resolve_old_path_within_retention() {
    assert!(
        false,
        "Wave 0 stub: old owner/name resolves via repository_redirects within retention (D-REL-08)"
    );
}

/// Live repo at old path supersedes redirect.
#[tokio::test]
#[ignore = "Wave 0: green in 15-03"]
async fn redirect_supersede_when_new_repo_occupies_old_path() {
    assert!(
        false,
        "Wave 0 stub: creating a repo at old path deletes matching redirect (live wins)"
    );
}

/// Expired redirects purge via job.
#[tokio::test]
#[ignore = "Wave 0: green in 15-03"]
async fn redirect_purge_expired_rows() {
    assert!(
        false,
        "Wave 0 stub: reconcile/purge deletes repository_redirects where expires_at < now (D-REL-08)"
    );
}

/// GIT-17 / D-REL-09 / D-REL-10: Admin transfer to user/org with type-confirm.
#[tokio::test]
#[ignore = "Wave 0: green in 15-04"]
async fn repo_transfer_admin_to_user_or_org_with_confirm() {
    assert!(
        false,
        "Wave 0 stub: Admin transfer to user/org after exact confirm_name; disk+owner rewrite + redirect (GIT-17, D-REL-09, D-REL-10)"
    );
}

/// D-REL-10: confirm_name mismatch rejected.
#[tokio::test]
#[ignore = "Wave 0: green in 15-04"]
async fn repo_transfer_confirm_mismatch() {
    assert!(
        false,
        "Wave 0 stub: confirm_name mismatch → transfer_confirm_mismatch / confirm_mismatch (D-REL-10)"
    );
}

/// D-REL-10: issues/LFS associations stay on repo_id (when tables exist).
#[tokio::test]
#[ignore = "Wave 0: green in 15-04"]
async fn repo_transfer_cascade_issues_lfs_by_repo_id() {
    assert!(
        false,
        "Wave 0 stub: transfer preserves issue/LFS association rows on repo_id; no OID copy (D-REL-10, D-REL-11)"
    );
}
