//! admin.instance.factory_reset scope radios (D-34): DB-only keeps disk; both wipes repos_dir.

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::auth::hash_password_str;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_db::Database;
use octanest_git::{CliGitBackend, GitBackend};
use tower::ServiceExt;
use uuid::Uuid;

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

async fn seed_sys_admin(db: &Database) -> String {
    let hash = hash_password_str("password1").expect("hash");
    let id = Uuid::new_v4().to_string();
    db.create_user(
        &id,
        "admin@ex.com",
        "adminuser",
        Some(&hash),
        "Admin",
        "",
        None,
        octanest_core::Role::SysAdmin,
    )
    .await
    .expect("create admin");
    id
}

async fn login_admin(app: &axum::Router) -> String {
    let login = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.login","input":{"identifier":"admin@ex.com","password":"password1","remember_me":false}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    session_cookie_from_response(&login)
}

#[tokio::test]
async fn factory_reset_database_only_keeps_repo_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("factory_reset_db_only.db").display()
    );
    let repos_dir = dir.path().join("repos");
    let bare = repos_dir.join("adminuser").join("keepme.git");
    tokio::fs::create_dir_all(&bare).await.unwrap();
    tokio::fs::write(bare.join("HEAD"), b"ref: refs/heads/main\n")
        .await
        .unwrap();

    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    seed_sys_admin(&db).await;

    let app = test_app(db, repos_dir.clone()).await;
    let cookie = login_admin(&app).await;

    let res = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.instance.factory_reset","input":{"confirmation":"RESET","scope":"database_only"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["data"]["needs_setup"], true);
    assert!(
        bare.join("HEAD").exists(),
        "database_only must keep bare repo files at {}",
        bare.display()
    );
}

#[tokio::test]
async fn factory_reset_database_and_repositories_wipes_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("factory_reset_both.db").display()
    );
    let repos_dir = dir.path().join("repos");
    let bare = repos_dir.join("adminuser").join("gone.git");
    tokio::fs::create_dir_all(&bare).await.unwrap();
    let git = CliGitBackend::new();
    git.init_bare(&bare, "main").await.expect("init_bare");

    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    seed_sys_admin(&db).await;

    let app = test_app(db, repos_dir.clone()).await;
    let cookie = login_admin(&app).await;

    let res = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.instance.factory_reset","input":{"confirmation":"RESET","scope":"database_and_repositories"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert!(repos_dir.exists(), "repos root must remain");
    assert!(
        !bare.exists(),
        "database_and_repositories must delete bare repo at {}",
        bare.display()
    );
}

#[tokio::test]
async fn factory_reset_defaults_scope_to_database_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("factory_reset_default.db").display()
    );
    let repos_dir = dir.path().join("repos");
    let bare = repos_dir.join("adminuser").join("default.git");
    tokio::fs::create_dir_all(&bare).await.unwrap();
    tokio::fs::write(bare.join("HEAD"), b"ref: refs/heads/main\n")
        .await
        .unwrap();

    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    seed_sys_admin(&db).await;

    let app = test_app(db, repos_dir).await;
    let cookie = login_admin(&app).await;

    // Omit scope — serde default database_only.
    let res = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.instance.factory_reset","input":{"confirmation":"RESET"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert!(bare.join("HEAD").exists(), "omitted scope keeps disk");
}

#[tokio::test]
async fn factory_reset_wrong_phrase_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("factory_reset_phrase.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    seed_sys_admin(&db).await;

    let app = test_app(db, dir.path().join("repos")).await;
    let cookie = login_admin(&app).await;

    let res = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.instance.factory_reset","input":{"confirmation":"reset","scope":"database_only"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "admin.factory_reset_confirm");
}
