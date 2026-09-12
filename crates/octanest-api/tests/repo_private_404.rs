//! GIT-05 / D-23–D-25: private non-owner and missing → identical `repo.not_found`;
//! anonymous can read public repos.

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

/// Missing repo and private non-owner must share `repo.not_found` (no existence leak).
#[tokio::test]
async fn repo_private_404_identical_not_found_for_missing_and_private() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!("sqlite:{}", dir.path().join("repo_private_404.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    // Owner creates a real private repository.
    let (owner_cookie, owner_v) = signup_and_login(&app, "owner@ex.com", "owner1").await;
    let owner_id = owner_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&owner_id, &now)
        .await
        .expect("verify owner");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"secret-private","visibility":"private","description":""}}"#,
            &owner_cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK, "private create must succeed");
    let create_bytes = create.into_body().collect().await.unwrap().to_bytes();
    let create_v: serde_json::Value = serde_json::from_slice(&create_bytes).unwrap();
    assert_eq!(create_v["ok"], true, "private create ok — {create_v}");

    // Stranger session (non-owner).
    let (stranger_cookie, _) = signup_and_login(&app, "stranger@ex.com", "stranger1").await;

    let missing = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.get","input":{"owner":"nobody","name":"missing-repo"}}"#,
            &stranger_cookie,
        ))
        .await
        .unwrap();
    let missing_bytes = missing.into_body().collect().await.unwrap().to_bytes();
    let missing_v: serde_json::Value = serde_json::from_slice(&missing_bytes).unwrap();

    let private = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.get","input":{"owner":"owner1","name":"secret-private"}}"#,
            &stranger_cookie,
        ))
        .await
        .unwrap();
    let private_bytes = private.into_body().collect().await.unwrap().to_bytes();
    let private_v: serde_json::Value = serde_json::from_slice(&private_bytes).unwrap();

    assert_eq!(
        missing_v["error"]["code"], "repo.not_found",
        "missing repo → repo.not_found — {missing_v}"
    );
    assert_eq!(
        private_v["error"]["code"], "repo.not_found",
        "private non-owner → repo.not_found (identical) — {private_v}"
    );
    assert_eq!(
        missing_v["error"]["code"], private_v["error"]["code"],
        "codes must match to avoid existence leak"
    );
    assert_eq!(
        missing_v["error"]["message"], private_v["error"]["message"],
        "messages must match to avoid existence leak"
    );
}

/// Anonymous callers can read public repo metadata (D-24).
#[tokio::test]
async fn repo_private_404_public_anonymous_get_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("repo_public_anon.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (owner_cookie, owner_v) = signup_and_login(&app, "pub@ex.com", "pubowner").await;
    let owner_id = owner_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&owner_id, &now)
        .await
        .expect("verify");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"open-src","visibility":"public","description":"hi"}}"#,
            &owner_cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let _ = create.into_body().collect().await;

    // No session cookie — anonymous.
    let get = app
        .oneshot(rpc_req(
            r#"{"procedure":"repo.get","input":{"owner":"pubowner","name":"open-src"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK, "anonymous public get must be 200");
    let bytes = get.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "anonymous public get ok — {v}");
    assert_eq!(v["data"]["name"], "open-src");
    assert_eq!(v["data"]["visibility"], "public");
    assert_eq!(v["data"]["owner_username"], "pubowner");
}

/// Empty public repo tree returns structured empty (no 500).
#[tokio::test]
async fn repo_private_404_empty_tree_structured() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repos = dir.path().join("repos");
    let url = format!(
        "sqlite:{}",
        dir.path().join("repo_empty_tree.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos).await;

    let (owner_cookie, owner_v) = signup_and_login(&app, "empty@ex.com", "emptyown").await;
    let owner_id = owner_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&owner_id, &now)
        .await
        .expect("verify");

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"repo.create","input":{"name":"blank","visibility":"public"}}"#,
            &owner_cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let _ = create.into_body().collect().await;

    let tree = app
        .oneshot(rpc_req(
            r#"{"procedure":"repo.tree","input":{"owner":"emptyown","name":"blank","ref":"main","path":""}}"#,
        ))
        .await
        .unwrap();
    assert_ne!(
        tree.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "empty tree must not 500"
    );
    let bytes = tree.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "empty tree ok — {v}");
    assert_eq!(v["data"]["empty"], true);
    assert_eq!(v["data"]["entries"], serde_json::json!([]));
}
