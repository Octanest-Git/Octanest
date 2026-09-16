//! Phase 17 — notification RPC integration (NOTF-01 / NOTF-02 / D-01 / D-03 / D-12 / T-17-01).

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

async fn create_repo(app: &axum::Router, cookie: &str, name: &str) {
    let body = format!(
        r#"{{"procedure":"repo.create","input":{{"name":"{name}","visibility":"public","description":""}}}}"#
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

async fn create_issue(app: &axum::Router, cookie: &str, owner: &str, name: &str, title: &str) {
    let body = format!(
        r#"{{"procedure":"issue.create","input":{{"owner":"{owner}","name":"{name}","title":"{title}","body":"root"}}}}"#
    );
    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(&body, cookie))
        .await
        .unwrap();
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "issue.create — {v}");
}

async fn rpc_json(app: &axum::Router, cookie: &str, body: &str) -> serde_json::Value {
    let res = app
        .clone()
        .oneshot(rpc_req_with_cookie(body, cookie))
        .await
        .unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn rpc_json_anon(app: &axum::Router, body: &str) -> serde_json::Value {
    let res = app.clone().oneshot(rpc_req(body)).await.unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn notification_list_empty_when_signed_in() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("notif_list_empty.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "nlist@ex.com", "nlist").await;
    verify_user(&db, login_v["data"]["id"].as_str().unwrap()).await;

    let listed = rpc_json(
        &app,
        &cookie,
        r#"{"procedure":"notification.list","input":{"filter":"all"}}"#,
    )
    .await;
    assert_eq!(listed["ok"], true, "{listed}");
    assert_eq!(listed["data"]["notifications"].as_array().unwrap().len(), 0);
    assert_eq!(listed["data"]["total"], 0);
}

#[tokio::test]
async fn notification_issue_comment_creates_unread_for_author() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("notif_comment.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (author_cookie, author_v) = signup_and_login(&app, "nauth@ex.com", "nauth").await;
    let author_id = author_v["data"]["id"].as_str().unwrap();
    verify_user(&db, author_id).await;
    create_repo(&app, &author_cookie, "talk").await;
    create_issue(&app, &author_cookie, "nauth", "talk", "Hello").await;

    let (writer_cookie, writer_v) = signup_and_login(&app, "nwriter@ex.com", "nwriter").await;
    let writer_id = writer_v["data"]["id"].as_str().unwrap();
    verify_user(&db, writer_id).await;
    let add = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"repo.collaborators.add","input":{"owner":"nauth","name":"talk","username":"nwriter","permission":"write"}}"#,
    )
    .await;
    assert_eq!(add["ok"], true, "{add}");

    let created = rpc_json(
        &app,
        &writer_cookie,
        r#"{"procedure":"issue.comments.create","input":{"owner":"nauth","name":"talk","number":1,"body":"ping"}}"#,
    )
    .await;
    assert_eq!(created["ok"], true, "{created}");

    let author_count = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(author_count["ok"], true, "{author_count}");
    assert_eq!(author_count["data"]["count"], 1);

    let writer_count = rpc_json(
        &app,
        &writer_cookie,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(writer_count["ok"], true, "{writer_count}");
    assert_eq!(writer_count["data"]["count"], 0);

    let listed = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.list","input":{"filter":"unread"}}"#,
    )
    .await;
    assert_eq!(listed["ok"], true, "{listed}");
    let rows = listed["data"]["notifications"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["reason"], "issue_comment");
    assert_eq!(rows[0]["subject_kind"], "issue");
    assert_eq!(rows[0]["subject_number"], 1);
    assert_eq!(rows[0]["owner"], "nauth");
    assert_eq!(rows[0]["repo"], "talk");
    assert_eq!(rows[0]["actor_username"], "nwriter");
}

#[tokio::test]
async fn notification_unread_count() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("notif_count.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "ncount@ex.com", "ncount").await;
    verify_user(&db, login_v["data"]["id"].as_str().unwrap()).await;
    let count = rpc_json(
        &app,
        &cookie,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(count["ok"], true, "{count}");
    assert_eq!(count["data"]["count"], 0);
}

#[tokio::test]
async fn notification_mark_read_and_mark_all_read() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("notif_mark.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (author_cookie, author_v) = signup_and_login(&app, "nmark@ex.com", "nmark").await;
    verify_user(&db, author_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &author_cookie, "marks").await;
    create_issue(&app, &author_cookie, "nmark", "marks", "T1").await;

    let (writer_cookie, writer_v) = signup_and_login(&app, "nmarkw@ex.com", "nmarkw").await;
    verify_user(&db, writer_v["data"]["id"].as_str().unwrap()).await;
    let add = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"repo.collaborators.add","input":{"owner":"nmark","name":"marks","username":"nmarkw","permission":"write"}}"#,
    )
    .await;
    assert_eq!(add["ok"], true, "{add}");

    for body in ["one", "two"] {
        let created = rpc_json(
            &app,
            &writer_cookie,
            &format!(
                r#"{{"procedure":"issue.comments.create","input":{{"owner":"nmark","name":"marks","number":1,"body":"{body}"}}}}"#
            ),
        )
        .await;
        assert_eq!(created["ok"], true, "{created}");
    }

    let listed = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.list","input":{"filter":"unread"}}"#,
    )
    .await;
    assert_eq!(listed["ok"], true, "{listed}");
    let rows = listed["data"]["notifications"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let first_id = rows[0]["id"].as_str().unwrap();

    let marked = rpc_json(
        &app,
        &author_cookie,
        &format!(
            r#"{{"procedure":"notification.markRead","input":{{"ids":["{first_id}"]}}}}"#
        ),
    )
    .await;
    assert_eq!(marked["ok"], true, "{marked}");
    assert_eq!(marked["data"]["marked"], 1);

    let mid = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(mid["data"]["count"], 1);

    let all = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.markAllRead","input":{}}"#,
    )
    .await;
    assert_eq!(all["ok"], true, "{all}");
    assert!(all["data"]["marked"].as_i64().unwrap() >= 1);

    let zero = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(zero["data"]["count"], 0);
}

#[tokio::test]
async fn notification_actor_is_not_notified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("notif_actor.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (cookie, login_v) = signup_and_login(&app, "nact@ex.com", "nact").await;
    verify_user(&db, login_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &cookie, "self").await;
    create_issue(&app, &cookie, "nact", "self", "Mine").await;
    let created = rpc_json(
        &app,
        &cookie,
        r#"{"procedure":"issue.comments.create","input":{"owner":"nact","name":"self","number":1,"body":"note to self"}}"#,
    )
    .await;
    assert_eq!(created["ok"], true, "{created}");
    let count = rpc_json(
        &app,
        &cookie,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(count["data"]["count"], 0);
}

#[tokio::test]
async fn notification_cannot_mark_another_users_notification() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("notif_idor.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (author_cookie, author_v) = signup_and_login(&app, "nidor@ex.com", "nidor").await;
    verify_user(&db, author_v["data"]["id"].as_str().unwrap()).await;
    create_repo(&app, &author_cookie, "idor").await;
    create_issue(&app, &author_cookie, "nidor", "idor", "Secret").await;

    let (writer_cookie, writer_v) = signup_and_login(&app, "nidorw@ex.com", "nidorw").await;
    verify_user(&db, writer_v["data"]["id"].as_str().unwrap()).await;
    let add = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"repo.collaborators.add","input":{"owner":"nidor","name":"idor","username":"nidorw","permission":"write"}}"#,
    )
    .await;
    assert_eq!(add["ok"], true, "{add}");
    let created = rpc_json(
        &app,
        &writer_cookie,
        r#"{"procedure":"issue.comments.create","input":{"owner":"nidor","name":"idor","number":1,"body":"hi"}}"#,
    )
    .await;
    assert_eq!(created["ok"], true, "{created}");

    let listed = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.list","input":{"filter":"unread"}}"#,
    )
    .await;
    let notif_id = listed["data"]["notifications"][0]["id"].as_str().unwrap();

    let steal = rpc_json(
        &app,
        &writer_cookie,
        &format!(
            r#"{{"procedure":"notification.markRead","input":{{"ids":["{notif_id}"]}}}}"#
        ),
    )
    .await;
    assert_eq!(steal["ok"], true, "{steal}");
    assert_eq!(steal["data"]["marked"], 0);

    let still = rpc_json(
        &app,
        &author_cookie,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(still["data"]["count"], 1);
}

#[tokio::test]
async fn notification_unauthenticated_fails_closed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("notif_auth.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db, repos).await;

    let list = rpc_json_anon(
        &app,
        r#"{"procedure":"notification.list","input":{}}"#,
    )
    .await;
    assert_eq!(list["ok"], false, "{list}");
    assert_eq!(list["error"]["code"], "auth.unauthenticated");

    let count = rpc_json_anon(
        &app,
        r#"{"procedure":"notification.unreadCount","input":{}}"#,
    )
    .await;
    assert_eq!(count["ok"], false, "{count}");
    assert_eq!(count["error"]["code"], "auth.unauthenticated");
}
