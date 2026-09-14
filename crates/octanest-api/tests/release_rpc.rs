//! Wave 0 / Phase 15: GIT-14/15 release + asset stubs (D-REL-01..06, D-REL-12).
//! Intentionally ignored until plans 15-01 / 15-02 turn them green.

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

async fn rpc_json(app: &axum::Router, body: &str, cookie: &str) -> serde_json::Value {
    let res = app
        .clone()
        .oneshot(rpc_req_with_cookie(body, cookie))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// GIT-14 / D-REL-01 / D-REL-02: release.create for existing tag with notes + draft/prerelease.
#[tokio::test]
#[ignore = "Wave 0: green in 15-01"]
async fn release_create_existing_tag_with_notes() {
    let _ = (test_app, signup_and_login, rpc_json);
    assert!(
        false,
        "Wave 0 stub: release.create must bind to existing tag with title/body/draft/prerelease (GIT-14, D-REL-01, D-REL-02)"
    );
}

/// D-REL-01: missing tag → release.tag_missing.
#[tokio::test]
#[ignore = "Wave 0: green in 15-01"]
async fn release_tag_missing_when_tag_absent() {
    assert!(
        false,
        "Wave 0 stub: release.create without refs/tags/{{tag}} returns release.tag_missing (D-REL-01)"
    );
}

/// D-REL-12: drafts hidden from Read/anon; visible to Write+.
#[tokio::test]
#[ignore = "Wave 0: green in 15-01"]
async fn release_draft_hidden_from_read_anon() {
    assert!(
        false,
        "Wave 0 stub: draft list/get invisible to anon and Read-only; visible to Write+ (D-REL-12)"
    );
}

/// D-REL-12: Write+ required for create/publish.
#[tokio::test]
#[ignore = "Wave 0: green in 15-01"]
async fn release_create_requires_write() {
    assert!(
        false,
        "Wave 0 stub: release.create requires meets(Write); soft repo.not_found otherwise (D-REL-12, T-15-01)"
    );
}

/// D-REL-03: Author or Write+ can update notes/flags.
#[tokio::test]
#[ignore = "Wave 0: green in 15-01"]
async fn release_update_notes_write_or_author() {
    assert!(
        false,
        "Wave 0 stub: release.update allows author or Write+ (D-REL-03)"
    );
}

/// D-REL-03 / D-REL-12: Admin delete.
#[tokio::test]
#[ignore = "Wave 0: green in 15-01"]
async fn release_delete_requires_admin() {
    assert!(
        false,
        "Wave 0 stub: release.delete requires Admin via resolve_repo_for_admin; soft not_found otherwise (D-REL-03)"
    );
}

/// GIT-15 / D-REL-04..06: asset upload stores under release-assets volume (not LFS).
#[tokio::test]
#[ignore = "Wave 0: green in 15-02"]
async fn release_asset_upload_download_acl() {
    assert!(
        false,
        "Wave 0 stub: multipart upload to OCTANEST_RELEASE_ASSETS_DIR/{{asset_id}}; GET AuthZ via repo ACL (GIT-15, D-REL-04, D-REL-06)"
    );
}

/// D-REL-05: oversized upload rejected.
#[tokio::test]
#[ignore = "Wave 0: green in 15-02"]
async fn release_asset_size_reject() {
    assert!(
        false,
        "Wave 0 stub: upload over OCTANEST_RELEASE_ASSET_MAX_BYTES rejected (D-REL-05)"
    );
}

/// D-REL-05: replace-on-edit allowed.
#[tokio::test]
#[ignore = "Wave 0: green in 15-02"]
async fn release_asset_replace_on_edit() {
    assert!(
        false,
        "Wave 0 stub: replacing an asset on edit updates bytes + metadata (D-REL-05)"
    );
}

/// D-REL-06: draft assets require Write+; private never anonymous.
#[tokio::test]
#[ignore = "Wave 0: green in 15-02"]
async fn release_asset_draft_and_private_acl() {
    assert!(
        false,
        "Wave 0 stub: draft assets Write+ only; private repos never anonymous download (D-REL-06, T-15-02)"
    );
}
