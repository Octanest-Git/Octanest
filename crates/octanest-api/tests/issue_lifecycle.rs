//! ISS-01 create/list/get/edit/close/reopen/history (D-ISS-01..04 / D-ISS-20).
//!
//! Hard-delete covered in `issue_delete.rs` (11-04-T2).

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

async fn verify_user(db: &Database, user_id: &str) {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(user_id, &now)
        .await
        .expect("verify");
}

async fn create_repo(app: &axum::Router, cookie: &str, name: &str, visibility: &str) {
    let body = format!(
        r#"{{"procedure":"repo.create","input":{{"name":"{name}","visibility":"{visibility}","description":""}}}}"#
    );
    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(&body, cookie))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "repo.create — {v}");
}

/// Verified Write+ can create an issue and receives per-repo `#1` (ISS-01 / D-ISS-01).
#[tokio::test]
async fn issue_lifecycle_create_allocates_per_repo_number() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("issue_create_n1.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "issowner@ex.com", "issowner").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;
    create_repo(&app, &cookie, "hello", "public").await;

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"issowner","name":"hello","title":"First","body":"body"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "issue.create ok — {v}");
    assert_eq!(v["data"]["number"], 1, "first issue is #1 — {v}");
    assert_eq!(v["data"]["title"], "First");
    assert_eq!(v["data"]["state"], "open");
    assert_eq!(v["data"]["author_username"], "issowner");

    let get = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.get","input":{"owner":"issowner","name":"hello","number":1}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let get_bytes = get.into_body().collect().await.unwrap().to_bytes();
    let get_v: serde_json::Value = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(get_v["ok"], true, "issue.get — {get_v}");
    assert_eq!(get_v["data"]["number"], 1);

    let list = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.list","input":{"owner":"issowner","name":"hello"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let list_bytes = list.into_body().collect().await.unwrap().to_bytes();
    let list_v: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    assert_eq!(list_v["ok"], true, "issue.list — {list_v}");
    assert_eq!(list_v["data"]["total"], 1);
    assert_eq!(list_v["data"]["issues"][0]["number"], 1);
}

/// Second create in same repo gets `#2`; second repo starts at `#1` (D-ISS-01).
#[tokio::test]
async fn issue_lifecycle_second_create_monotonic_number() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("issue_create_n2.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "mono@ex.com", "monoown").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;
    create_repo(&app, &cookie, "repo-a", "public").await;
    create_repo(&app, &cookie, "repo-b", "public").await;

    let c1 = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"monoown","name":"repo-a","title":"A1"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let c1b = c1.into_body().collect().await.unwrap().to_bytes();
    let c1v: serde_json::Value = serde_json::from_slice(&c1b).unwrap();
    assert_eq!(c1v["ok"], true, "{c1v}");
    assert_eq!(c1v["data"]["number"], 1);

    let c2 = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"monoown","name":"repo-a","title":"A2"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let c2b = c2.into_body().collect().await.unwrap().to_bytes();
    let c2v: serde_json::Value = serde_json::from_slice(&c2b).unwrap();
    assert_eq!(c2v["ok"], true, "{c2v}");
    assert_eq!(c2v["data"]["number"], 2, "second in same repo is #2");

    let b1 = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"monoown","name":"repo-b","title":"B1"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let b1b = b1.into_body().collect().await.unwrap().to_bytes();
    let b1v: serde_json::Value = serde_json::from_slice(&b1b).unwrap();
    assert_eq!(b1v["ok"], true, "{b1v}");
    assert_eq!(b1v["data"]["number"], 1, "second repo starts at #1");
}

/// Author or Write+ may edit title/body after create (ISS-01 / D-ISS-03).
#[tokio::test]
async fn issue_lifecycle_edit_title_body() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("issue_edit.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (owner_cookie, login_v) = signup_and_login(&app, "editown@ex.com", "editown").await;
    let owner_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, owner_id).await;
    create_repo(&app, &owner_cookie, "edits", "public").await;

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"editown","name":"edits","title":"Original","body":"v1"}}"#,
            &owner_cookie,
        ))
        .await
        .unwrap();
    let create_b = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_b).unwrap();
    assert_eq!(create_v["ok"], true, "{create_v}");

    let update = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.update","input":{"owner":"editown","name":"edits","number":1,"title":"Edited","body":"v2"}}"#,
            &owner_cookie,
        ))
        .await
        .unwrap();
    let update_b = update.into_body().collect().await.unwrap().to_bytes();
    let update_v: serde_json::Value = serde_json::from_slice(&update_b).unwrap();
    assert_eq!(update_v["ok"], true, "author can update — {update_v}");
    assert_eq!(update_v["data"]["title"], "Edited");
    assert_eq!(update_v["data"]["body"], "v2");

    // Read collaborator cannot update (D-ISS-03 / D-ISS-20).
    let (reader_cookie, reader_v) =
        signup_and_login(&app, "editread@ex.com", "editread").await;
    let reader_id = reader_v["data"]["id"].as_str().expect("id");
    verify_user(&db, reader_id).await;
    let add = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.collaborators.add","input":{"owner":"editown","name":"edits","username":"editread","permission":"read"}}"#,
            &owner_cookie,
        ))
        .await
        .unwrap();
    let add_b = add.into_body().collect().await.unwrap().to_bytes();
    let add_v: serde_json::Value = serde_json::from_slice(&add_b).unwrap();
    assert_eq!(add_v["ok"], true, "add read collab — {add_v}");

    let denied = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.update","input":{"owner":"editown","name":"edits","number":1,"title":"Nope"}}"#,
            &reader_cookie,
        ))
        .await
        .unwrap();
    let denied_b = denied.into_body().collect().await.unwrap().to_bytes();
    let denied_v: serde_json::Value = serde_json::from_slice(&denied_b).unwrap();
    assert_eq!(denied_v["ok"], false, "read cannot update — {denied_v}");
    assert_eq!(
        denied_v["error"]["code"], "repo.not_found",
        "soft deny — {denied_v}"
    );
}

/// Lifecycle is open ↔ closed; reopen allowed (ISS-01 / D-ISS-02).
#[tokio::test]
async fn issue_lifecycle_close_and_reopen() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("issue_close.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "closeown@ex.com", "closeown").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;
    create_repo(&app, &cookie, "cycle", "public").await;

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"closeown","name":"cycle","title":"Toggle me"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let create_b = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_b).unwrap();
    assert_eq!(create_v["ok"], true, "{create_v}");
    assert_eq!(create_v["data"]["state"], "open");

    let close = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.close","input":{"owner":"closeown","name":"cycle","number":1}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let close_b = close.into_body().collect().await.unwrap().to_bytes();
    let close_v: serde_json::Value = serde_json::from_slice(&close_b).unwrap();
    assert_eq!(close_v["ok"], true, "close — {close_v}");
    assert_eq!(close_v["data"]["state"], "closed");
    assert!(close_v["data"]["closed_at"].as_str().is_some());

    let reopen = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.reopen","input":{"owner":"closeown","name":"cycle","number":1}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let reopen_b = reopen.into_body().collect().await.unwrap().to_bytes();
    let reopen_v: serde_json::Value = serde_json::from_slice(&reopen_b).unwrap();
    assert_eq!(reopen_v["ok"], true, "reopen — {reopen_v}");
    assert_eq!(reopen_v["data"]["state"], "open");
    assert!(
        reopen_v["data"]["closed_at"].is_null()
            || reopen_v["data"].get("closed_at").is_none()
    );
}

/// Full edit history trail for title/body (ISS-01 / D-ISS-04).
#[tokio::test]
async fn issue_history_full_title_body_trail() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("issue_hist.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "histown@ex.com", "histown").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;
    create_repo(&app, &cookie, "trail", "public").await;

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.create","input":{"owner":"histown","name":"trail","title":"T0","body":"B0"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let create_b = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_b).unwrap();
    assert_eq!(create_v["ok"], true, "{create_v}");

    for (title, body) in [("T1", "B1"), ("T2", "B2")] {
        let body_json = format!(
            r#"{{"procedure":"issue.update","input":{{"owner":"histown","name":"trail","number":1,"title":"{title}","body":"{body}"}}}}"#
        );
        let upd = app
            .clone()
            .oneshot(rpc_req_with_cookie(&body_json, &cookie))
            .await
            .unwrap();
        let upd_b = upd.into_body().collect().await.unwrap().to_bytes();
        let upd_v: serde_json::Value = serde_json::from_slice(&upd_b).unwrap();
        assert_eq!(upd_v["ok"], true, "update {title} — {upd_v}");
    }

    let hist = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"issue.history","input":{"owner":"histown","name":"trail","number":1}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    let hist_b = hist.into_body().collect().await.unwrap().to_bytes();
    let hist_v: serde_json::Value = serde_json::from_slice(&hist_b).unwrap();
    assert_eq!(hist_v["ok"], true, "history — {hist_v}");
    let revs = hist_v["data"]["revisions"]
        .as_array()
        .expect("revisions array");
    assert_eq!(revs.len(), 2, "two prior snapshots — {hist_v}");
    assert_eq!(revs[0]["title"], "T0");
    assert_eq!(revs[0]["body"], "B0");
    assert_eq!(revs[1]["title"], "T1");
    assert_eq!(revs[1]["body"], "B1");
    assert_eq!(revs[0]["editor_username"], "histown");
}
