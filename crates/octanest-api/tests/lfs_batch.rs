//! Wave 1 tracer: LFS Batch + basic PUT/GET into OCTANEST_LFS_DIR (GIT-12 / D-LFS-07).

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::auth::session::sha256_hex;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_db::Database;
use tower::ServiceExt;

async fn test_app(
    db: Database,
    repos_dir: std::path::PathBuf,
    lfs_dir: std::path::PathBuf,
) -> axum::Router {
    let state = AppState::new(db, Arc::new(LogSink) as Arc<dyn EmailSender>, "development")
        .with_repos_dir(repos_dir)
        .with_lfs_dir(lfs_dir);
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

fn encode_b64(input: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let mut n = (chunk[0] as u32) << 16;
        if chunk.len() > 1 {
            n |= (chunk[1] as u32) << 8;
        }
        if chunk.len() > 2 {
            n |= chunk[2] as u32;
        }
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn basic_header(user: &str, password: &str) -> String {
    let raw = format!("{user}:{password}");
    format!("Basic {}", encode_b64(raw.as_bytes()))
}

/// GIT-12: POST batch upload → PUT → GET download happy path (basic transfer).
#[tokio::test]
async fn lfs_batch_upload_put_download_happy_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let lfs = dir.path().join("lfs");
    let url = format!("sqlite:{}", dir.path().join("lfs_happy.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos, lfs.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "lfs@ex.com", "lfsown").await;
    let user_id = login_v["data"]["id"].as_str().unwrap();
    verify_user(&db, user_id).await;

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"blobs","visibility":"public","description":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let create_bytes = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_bytes).unwrap();
    let repo_id = create_v["data"]["id"].as_str().expect("repo id");
    db.set_repo_lfs_enabled(repo_id, true)
        .await
        .expect("enable lfs");

    let create_pat = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"pat.createClassic","input":{"name":"lfs","scopes":["repo"]}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create_pat.status(), StatusCode::OK);
    let pat_bytes = create_pat.into_body().collect().await.unwrap().to_bytes();
    let pat_v: serde_json::Value = serde_json::from_slice(&pat_bytes).unwrap();
    let token = pat_v["data"]["token"].as_str().expect("token");

    let payload = b"hello-lfs-object";
    let oid = sha256_hex(payload);
    let size = payload.len() as i64;

    let batch_body = serde_json::json!({
        "operation": "upload",
        "transfers": ["basic"],
        "objects": [{ "oid": oid, "size": size }]
    });
    let batch_req = Request::builder()
        .method("POST")
        .uri("/lfsown/blobs.git/info/lfs/objects/batch")
        .header(header::AUTHORIZATION, basic_header("git", token))
        .header(header::ACCEPT, "application/vnd.git-lfs+json")
        .header(header::CONTENT_TYPE, "application/vnd.git-lfs+json")
        .body(Body::from(batch_body.to_string()))
        .unwrap();
    let batch_res = app.clone().oneshot(batch_req).await.unwrap();
    assert_eq!(batch_res.status(), StatusCode::OK, "batch upload");
    let batch_bytes = batch_res.into_body().collect().await.unwrap().to_bytes();
    let batch_v: serde_json::Value = serde_json::from_slice(&batch_bytes).unwrap();
    assert_eq!(batch_v["transfer"], "basic");
    let href = batch_v["objects"][0]["actions"]["upload"]["href"]
        .as_str()
        .expect("upload href");
    assert!(href.contains(&oid), "href={href}");

    let put_req = Request::builder()
        .method("PUT")
        .uri(href)
        .header(header::AUTHORIZATION, basic_header("git", token))
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(payload.as_slice()))
        .unwrap();
    let put_res = app.clone().oneshot(put_req).await.unwrap();
    assert_eq!(put_res.status(), StatusCode::OK, "put object");

    let shard = lfs.join(&oid[0..2]).join(&oid[2..4]).join(&oid);
    assert!(shard.is_file(), "shard missing at {}", shard.display());
    let on_disk = std::fs::read(&shard).unwrap();
    assert_eq!(on_disk, payload);

    let dl_batch = serde_json::json!({
        "operation": "download",
        "transfers": ["basic"],
        "objects": [{ "oid": oid, "size": size }]
    });
    let dl_req = Request::builder()
        .method("POST")
        .uri("/lfsown/blobs.git/info/lfs/objects/batch")
        .header(header::AUTHORIZATION, basic_header("git", token))
        .header(header::ACCEPT, "application/vnd.git-lfs+json")
        .header(header::CONTENT_TYPE, "application/vnd.git-lfs+json")
        .body(Body::from(dl_batch.to_string()))
        .unwrap();
    let dl_res = app.clone().oneshot(dl_req).await.unwrap();
    assert_eq!(dl_res.status(), StatusCode::OK);
    let dl_bytes = dl_res.into_body().collect().await.unwrap().to_bytes();
    let dl_v: serde_json::Value = serde_json::from_slice(&dl_bytes).unwrap();
    let dl_href = dl_v["objects"][0]["actions"]["download"]["href"]
        .as_str()
        .expect("download href");

    let get_req = Request::builder()
        .method("GET")
        .uri(dl_href)
        .header(header::AUTHORIZATION, basic_header("git", token))
        .body(Body::empty())
        .unwrap();
    let get_res = app.oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
    let got = get_res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&got[..], payload);
}

async fn verify_user(db: &Database, user_id: &str) {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(user_id, &now)
        .await
        .expect("verify email");
}

/// Placeholder auth stubs remain discoverable (greened in 14-03).
#[tokio::test]
async fn lfs_batch_cookie_ignored_as_anon() {}

#[tokio::test]
async fn lfs_batch_unauth_private_returns_401_lfs_authenticate() {}

#[tokio::test]
async fn lfs_batch_insufficient_pat_scope_forbidden() {}

#[tokio::test]
async fn lfs_batch_upload_requires_write_and_verified() {}

#[tokio::test]
async fn lfs_enable_disabled_repo_rejects_batch() {}

#[tokio::test]
async fn lfs_quota_over_quota_upload_rejected() {}

#[tokio::test]
async fn lfs_dedup_existing_oid_omits_upload_actions() {}
