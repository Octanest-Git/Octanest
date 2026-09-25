//! GIT-07 / D-29: streaming zip + tar.gz archives after ACL (not RPC JSON).

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oxidean_api::email::{EmailSender, LogSink};
use oxidean_api::{build_cors, router_with_state, AppState};
use oxidean_db::Database;
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
        .header("Oxidean-RPC-Version", "1")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn rpc_req_with_cookie(body: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/rpc")
        .header("content-type", "application/json")
        .header("Oxidean-RPC-Version", "1")
        .header("cookie", cookie)
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn archive_req(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(c) = cookie {
        builder = builder.header("cookie", c);
    }
    builder.body(Body::empty()).unwrap()
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

/// Seeded ref → non-empty zip and tar.gz bytes (GIT-07).
#[tokio::test]
async fn repo_archive_zip_and_tar_gz_nonempty_for_seeded_ref() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("repo_archive_seed.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "arch@ex.com", "archowner").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"archived","visibility":"public","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let create_bytes = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_bytes).unwrap();
    assert_eq!(create_v["ok"], true, "create — {create_v}");

    let zip = app
        .clone()
        .oneshot(archive_req(
            "/api/repos/archowner/archived/archive/main.zip",
            Some(&cookie),
        ))
        .await
        .unwrap();
    assert_eq!(zip.status(), StatusCode::OK, "zip status");
    let zip_bytes = zip.into_body().collect().await.unwrap().to_bytes();
    assert!(!zip_bytes.is_empty(), "zip must be non-empty");
    assert_eq!(&zip_bytes[0..2], b"PK", "zip magic");

    let tar = app
        .clone()
        .oneshot(archive_req(
            "/api/repos/archowner/archived/archive/main.tar.gz",
            Some(&cookie),
        ))
        .await
        .unwrap();
    assert_eq!(tar.status(), StatusCode::OK, "tar.gz status");
    let tar_bytes = tar.into_body().collect().await.unwrap().to_bytes();
    assert!(!tar_bytes.is_empty(), "tar.gz must be non-empty");
    assert_eq!(&tar_bytes[0..2], &[0x1f, 0x8b], "gzip magic");
}

/// Private non-owner → same not-found surface as missing (no leak).
#[tokio::test]
async fn repo_archive_private_non_owner_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("repo_archive_private.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (owner_cookie, owner_v) = signup_and_login(&app, "ownarch@ex.com", "ownarch").await;
    let owner_id = owner_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&owner_id, &now)
        .await
        .expect("verify owner");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"secret-arch","visibility":"private","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}"#,
            &owner_cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let _ = create.into_body().collect().await;

    let (stranger_cookie, _) = signup_and_login(&app, "strarch@ex.com", "strarch").await;

    let denied = app
        .clone()
        .oneshot(archive_req(
            "/api/repos/ownarch/secret-arch/archive/main.zip",
            Some(&stranger_cookie),
        ))
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::NOT_FOUND);
    let denied_bytes = denied.into_body().collect().await.unwrap().to_bytes();
    let denied_v: serde_json::Value = serde_json::from_slice(&denied_bytes).unwrap();
    assert_eq!(denied_v["ok"], false);
    assert_eq!(denied_v["error"]["code"], "repo.not_found");

    let missing = app
        .clone()
        .oneshot(archive_req(
            "/api/repos/ownarch/no-such-repo/archive/main.zip",
            Some(&stranger_cookie),
        ))
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    let missing_bytes = missing.into_body().collect().await.unwrap().to_bytes();
    let missing_v: serde_json::Value = serde_json::from_slice(&missing_bytes).unwrap();
    assert_eq!(missing_v["error"]["code"], "repo.not_found");
}

/// Empty repo → structured failure (404/409), not 500 panic.
#[tokio::test]
async fn repo_archive_empty_repo_structured_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("repo_archive_empty.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "emptyarch@ex.com", "emptyarch").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"blank","visibility":"public","description":""}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let create_bytes = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_bytes).unwrap();
    assert_eq!(create_v["ok"], true, "empty create — {create_v}");

    let res = app
        .clone()
        .oneshot(archive_req(
            "/api/repos/emptyarch/blank/archive/main.zip",
            Some(&cookie),
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::NOT_FOUND || res.status() == StatusCode::CONFLICT,
        "empty archive must be structured 404/409, got {}",
        res.status()
    );
    assert_ne!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], false);
    assert!(
        v["error"]["code"].as_str().is_some(),
        "structured error code required — {v}"
    );
}

/// CR-01 / GIT-07: option-like archive treeish must not create/truncate files via git --output=.
#[tokio::test]
async fn repo_archive_rejects_option_like_treeish_no_output_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let monitored = dir.path().join("cr01_pwned_output.zip");
    assert!(
        !monitored.exists(),
        "monitored path must not exist before request"
    );

    let url = format!(
        "sqlite:{}",
        dir.path().join("repo_archive_cr01.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "cr01@ex.com", "cr01owner").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"cr01pub","visibility":"public","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let create_bytes = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_bytes).unwrap();
    assert_eq!(create_v["ok"], true, "create — {create_v}");

    // Treeish path segment: --output=<monitored> — encode `/` so it stays one path segment.
    let treeish = format!("--output={}", monitored.display());
    let encoded = treeish.replace('/', "%2F");
    let archive_uri = format!("/api/repos/cr01owner/cr01pub/archive/{encoded}.zip");

    let res = app
        .clone()
        .oneshot(archive_req(&archive_uri, Some(&cookie)))
        .await
        .unwrap();
    assert!(
        res.status().is_client_error(),
        "option-like treeish must be 4xx, got {}",
        res.status()
    );
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], false, "must fail — {v}");
    assert_eq!(
        v["error"]["code"], "repo.invalid_ref",
        "option-like treeish must be invalid_ref — {v}"
    );
    // HTTP-boundary reject (validate_archive_treeish), not only CLI validate_treeish (CR-01 defense-in-depth).
    assert_eq!(
        v["error"]["message"].as_str(),
        Some("invalid ref"),
        "must reject at HTTP validate_archive_treeish — {v}"
    );
    assert!(
        !monitored.exists(),
        "CR-01: monitored --output path must not exist after request"
    );
}
