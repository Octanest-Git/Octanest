//! Phase 15: GIT-14 release RPC (D-REL-01..03, D-REL-12). Asset tests remain ignored until 15-02.

mod support;

use std::process::Command;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_db::Database;
use octanest_git::{CliGitBackend, GitBackend};
use tower::ServiceExt;

async fn test_app(db: Database, repos_dir: std::path::PathBuf) -> axum::Router {
    let state = AppState::new(db, Arc::new(LogSink) as Arc<dyn EmailSender>, "development")
        .with_repos_dir(repos_dir)
        .with_git(Arc::new(CliGitBackend::new()));
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
    let set_cookie = res.headers().get("set-cookie").expect("Set-Cookie").to_str().unwrap();
    set_cookie.split(';').next().unwrap().trim().to_string()
}

async fn signup_and_login(app: &axum::Router, email: &str, username: &str) -> (String, serde_json::Value) {
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
    let res = app.clone().oneshot(rpc_req_with_cookie(body, cookie)).await.unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).expect("rpc json body")
}

async fn setup_repo_with_tag(app: &axum::Router, db: &Database, repos: &std::path::Path, cookie: &str, owner: &str, repo: &str, tag: &str) {
    let create = rpc_json(
        app,
        &format!(r#"{{"procedure":"repo.create","input":{{"name":"{repo}","visibility":"public","description":""}}}}"#),
        cookie,
    ).await;
    assert_eq!(create["ok"], true, "{create}");
    let bare = repos.join(owner).join(format!("{repo}.git"));
    let git = CliGitBackend::new();
    git.seed_commit(&bare, "main", "seed", &[("README.md".into(), b"hi".to_vec())])
        .await
        .expect("seed");
    let status = Command::new("git").args(["-C", bare.to_str().unwrap(), "tag", tag]).status().expect("tag");
    assert!(status.success(), "git tag failed");
    let _ = db;
}

#[tokio::test]
async fn release_create_existing_tag_with_notes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("rel_create.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;
    let (cookie, login_v) = signup_and_login(&app, "owner@ex.com", "owner1").await;
    let user_id = login_v["data"]["id"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now).await.expect("verify");

    setup_repo_with_tag(&app, &db, &repos, &cookie, "owner1", "hello", "v1.0.0").await;

    let created = rpc_json(
        &app,
        r#"{"procedure":"release.create","input":{"owner":"owner1","name":"hello","tag_name":"v1.0.0","title":"One","body":"notes here","draft":false,"prerelease":true}}"#,
        &cookie,
    ).await;
    assert_eq!(created["ok"], true, "{created}");
    assert_eq!(created["data"]["tag_name"], "v1.0.0");
    assert_eq!(created["data"]["title"], "One");
    assert_eq!(created["data"]["body"], "notes here");
    assert_eq!(created["data"]["prerelease"], true);
    assert_eq!(created["data"]["draft"], false);

    let listed = rpc_json(
        &app,
        r#"{"procedure":"release.list","input":{"owner":"owner1","name":"hello"}}"#,
        &cookie,
    ).await;
    assert_eq!(listed["ok"], true, "{listed}");
    assert_eq!(listed["data"]["releases"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn release_tag_missing_when_tag_absent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("rel_missing.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;
    let (cookie, login_v) = signup_and_login(&app, "owner2@ex.com", "owner2").await;
    let user_id = login_v["data"]["id"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now).await.expect("verify");
    setup_repo_with_tag(&app, &db, &repos, &cookie, "owner2", "hello", "v1.0.0").await;

    let missing = rpc_json(
        &app,
        r#"{"procedure":"release.create","input":{"owner":"owner2","name":"hello","tag_name":"nope","title":"x","body":""}}"#,
        &cookie,
    ).await;
    assert_eq!(missing["ok"], false, "{missing}");
    assert_eq!(missing["error"]["code"], "release.tag_missing");
}

#[tokio::test]
async fn release_create_requires_write() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("rel_write.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;
    let (cookie, login_v) = signup_and_login(&app, "owner3@ex.com", "owner3").await;
    let user_id = login_v["data"]["id"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now).await.expect("verify");
    setup_repo_with_tag(&app, &db, &repos, &cookie, "owner3", "hello", "v1.0.0").await;

    let (reader_cookie, reader_v) = signup_and_login(&app, "reader@ex.com", "reader1").await;
    let reader_id = reader_v["data"]["id"].as_str().unwrap().to_string();
    db.set_email_verified_at(&reader_id, &now).await.expect("verify reader");

    let denied = rpc_json(
        &app,
        r#"{"procedure":"release.create","input":{"owner":"owner3","name":"hello","tag_name":"v1.0.0","title":"x","body":""}}"#,
        &reader_cookie,
    ).await;
    assert_eq!(denied["ok"], false, "{denied}");
    assert_eq!(denied["error"]["code"], "repo.not_found");
}

#[tokio::test]
async fn release_draft_hidden_from_read_anon() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("rel_draft.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;
    let (cookie, login_v) = signup_and_login(&app, "owner4@ex.com", "owner4").await;
    let user_id = login_v["data"]["id"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now).await.expect("verify");
    setup_repo_with_tag(&app, &db, &repos, &cookie, "owner4", "hello", "v2.0.0").await;

    let created = rpc_json(
        &app,
        r#"{"procedure":"release.create","input":{"owner":"owner4","name":"hello","tag_name":"v2.0.0","title":"Draft","body":"secret","draft":true,"prerelease":false}}"#,
        &cookie,
    ).await;
    assert_eq!(created["ok"], true, "{created}");

    let anon_list = app
        .clone()
        .oneshot(rpc_req(r#"{"procedure":"release.list","input":{"owner":"owner4","name":"hello"}}"#))
        .await
        .unwrap();
    assert_eq!(anon_list.status(), StatusCode::OK);
    let bytes = anon_list.into_body().collect().await.unwrap().to_bytes();
    let listed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(listed["ok"], true, "{listed}");
    assert!(listed["data"]["releases"].as_array().unwrap().is_empty());

    let owner_list = rpc_json(
        &app,
        r#"{"procedure":"release.list","input":{"owner":"owner4","name":"hello"}}"#,
        &cookie,
    ).await;
    assert_eq!(owner_list["data"]["releases"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn release_update_notes_write_or_author() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("rel_update.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;
    let (cookie, login_v) = signup_and_login(&app, "owner5@ex.com", "owner5").await;
    let user_id = login_v["data"]["id"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now).await.expect("verify");
    setup_repo_with_tag(&app, &db, &repos, &cookie, "owner5", "hello", "v3.0.0").await;
    let _ = rpc_json(
        &app,
        r#"{"procedure":"release.create","input":{"owner":"owner5","name":"hello","tag_name":"v3.0.0","title":"Old","body":"a"}}"#,
        &cookie,
    ).await;
    let updated = rpc_json(
        &app,
        r#"{"procedure":"release.update","input":{"owner":"owner5","name":"hello","tag_name":"v3.0.0","title":"New","body":"b","draft":false}}"#,
        &cookie,
    ).await;
    assert_eq!(updated["ok"], true, "{updated}");
    assert_eq!(updated["data"]["title"], "New");
    assert_eq!(updated["data"]["body"], "b");
}

#[tokio::test]
async fn release_delete_requires_admin() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("rel_delete.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;
    let (cookie, login_v) = signup_and_login(&app, "owner6@ex.com", "owner6").await;
    let user_id = login_v["data"]["id"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now).await.expect("verify");
    setup_repo_with_tag(&app, &db, &repos, &cookie, "owner6", "hello", "v4.0.0").await;
    let _ = rpc_json(
        &app,
        r#"{"procedure":"release.create","input":{"owner":"owner6","name":"hello","tag_name":"v4.0.0","title":"X","body":""}}"#,
        &cookie,
    ).await;
    let deleted = rpc_json(
        &app,
        r#"{"procedure":"release.delete","input":{"owner":"owner6","name":"hello","tag_name":"v4.0.0"}}"#,
        &cookie,
    ).await;
    assert_eq!(deleted["ok"], true, "{deleted}");
    let listed = rpc_json(
        &app,
        r#"{"procedure":"release.list","input":{"owner":"owner6","name":"hello"}}"#,
        &cookie,
    ).await;
    assert!(listed["data"]["releases"].as_array().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "Wave 0 asset tests green in 15-02"]
async fn release_asset_upload_download_acl() {
    assert!(false, "15-02");
}

#[tokio::test]
#[ignore = "Wave 0 asset tests green in 15-02"]
async fn release_asset_size_reject() {
    assert!(false, "15-02");
}

#[tokio::test]
#[ignore = "Wave 0 asset tests green in 15-02"]
async fn release_asset_replace_on_edit() {
    assert!(false, "15-02");
}

#[tokio::test]
#[ignore = "Wave 0 asset tests green in 15-02"]
async fn release_asset_draft_and_private_acl() {
    assert!(false, "15-02");
}
