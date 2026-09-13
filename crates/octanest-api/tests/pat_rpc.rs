//! GIT-11: pat.createClassic / list / revoke + verified gate (08-04 tracer).

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_core::CLASSIC_PAT_PREFIX;
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

async fn rpc_json(app: &axum::Router, body: &str, cookie: &str) -> (StatusCode, serde_json::Value) {
    let res = app
        .clone()
        .oneshot(rpc_req_with_cookie(body, cookie))
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (status, v)
}

/// Verified `pat.createClassic` returns a one-time plaintext `token` field (D-15).
#[tokio::test]
async fn pat_create_classic_returns_one_time_token() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_create.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "pat@ex.com", "patuser").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let (status, v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createClassic","input":{"name":"laptop","scopes":["repo"]}}"#,
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "createClassic — {v}");
    assert_eq!(v["ok"], true, "{v}");
    let token = v["data"]["token"].as_str().expect("token");
    assert!(
        token.starts_with(CLASSIC_PAT_PREFIX),
        "must mint {CLASSIC_PAT_PREFIX}* — got {token}"
    );
    assert_eq!(v["data"]["item"]["kind"], "classic");
    assert_eq!(v["data"]["item"]["name"], "laptop");
    assert_eq!(v["data"]["item"]["token_prefix"], CLASSIC_PAT_PREFIX);
    assert!(v["data"]["item"].get("token").is_none());
}

/// `pat.list` never returns plaintext token secrets (D-15 / T-08-01).
#[tokio::test]
async fn pat_list_omits_secret_token() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_list.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "list@ex.com", "listuser").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let (_, create_v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createClassic","input":{"name":"ci","scopes":["repo"]}}"#,
        &cookie,
    )
    .await;
    assert_eq!(create_v["ok"], true, "{create_v}");
    let plaintext = create_v["data"]["token"].as_str().unwrap().to_string();

    let (status, list_v) = rpc_json(&app, r#"{"procedure":"pat.list","input":{}}"#, &cookie).await;
    assert_eq!(status, StatusCode::OK, "{list_v}");
    assert_eq!(list_v["ok"], true);
    let items = list_v["data"].as_array().expect("list array");
    assert_eq!(items.len(), 1);
    assert!(items[0].get("token").is_none(), "list must omit secret");
    let dumped = list_v.to_string();
    assert!(
        !dumped.contains(&plaintext),
        "plaintext must not appear in list response"
    );
    assert_eq!(items[0]["token_prefix"], CLASSIC_PAT_PREFIX);
}

/// `pat.revoke` removes the token from subsequent list results.
#[tokio::test]
async fn pat_revoke_removes_from_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_revoke.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "rev@ex.com", "revuser").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let (_, create_v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createClassic","input":{"name":"temp","scopes":["repo"]}}"#,
        &cookie,
    )
    .await;
    let id = create_v["data"]["item"]["id"].as_str().unwrap();

    let (status, rev_v) = rpc_json(
        &app,
        &format!(r#"{{"procedure":"pat.revoke","input":{{"id":"{id}"}}}}"#),
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{rev_v}");
    assert_eq!(rev_v["ok"], true);

    let (_, list_v) = rpc_json(&app, r#"{"procedure":"pat.list","input":{}}"#, &cookie).await;
    let items = list_v["data"].as_array().expect("list");
    assert!(
        items.iter().all(|i| i["id"] != id),
        "revoked id must be absent — {list_v}"
    );
}

/// Unverified session cannot create PATs → `auth.email_unverified` (D-24).
#[tokio::test]
async fn pat_create_unverified_email_unverified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_unverified.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db).await;

    let (cookie, login_v) = signup_and_login(&app, "newbie@ex.com", "newbie1").await;
    assert_eq!(login_v["data"]["email_verified"], false);

    let (status, v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createClassic","input":{"name":"x","scopes":["repo"]}}"#,
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{v}");
    assert_eq!(v["error"]["code"], "auth.email_unverified");
}

/// Empty / whitespace note on create → `pat.note_required` (D-16).
#[tokio::test]
async fn pat_create_empty_note_required() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_note.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "note@ex.com", "noteuser").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let (status, v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createClassic","input":{"name":"   ","scopes":["repo"]}}"#,
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{v}");
    assert_eq!(v["error"]["code"], "pat.note_required");
}
