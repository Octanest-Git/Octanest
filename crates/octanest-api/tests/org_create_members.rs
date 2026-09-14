//! ORG-01 / ORG-02: org.create happy path + deferred Wave 0 stubs for members/ACL.

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

/// Verified `org.create` reserves a slug shared with the user namespace (D-ORG-01).
#[tokio::test]
async fn org_create_reserves_shared_slug_namespace() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("org_create_slug.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), dir.path().join("repos")).await;

    let (cookie, login_v) = signup_and_login(&app, "owner@ex.com", "owner1").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;

    // Collision with an existing username must fail (shared namespace).
    let collide = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"org.create","input":{"slug":"owner1"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(collide.status(), StatusCode::BAD_REQUEST);
    let collide_bytes = collide.into_body().collect().await.unwrap().to_bytes();
    let collide_v: serde_json::Value = serde_json::from_slice(&collide_bytes).unwrap();
    assert_eq!(collide_v["ok"], false);
    assert_eq!(collide_v["error"]["code"], "org.slug_taken");

    // Happy path: unique slug succeeds.
    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"org.create","input":{"slug":"acme-labs","display_name":"Acme Labs"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK, "org.create must succeed");
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "org.create ok=true — {v}");
    assert_eq!(v["data"]["slug"], "acme-labs");
    assert_eq!(v["data"]["display_name"], "Acme Labs");
    assert_eq!(v["data"]["member_base_permission"], "none");

    // Second create with same slug fails (org↔org uniqueness).
    let dup = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"org.create","input":{"slug":"acme-labs"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(dup.status(), StatusCode::BAD_REQUEST);
    let dup_bytes = dup.into_body().collect().await.unwrap().to_bytes();
    let dup_v: serde_json::Value = serde_json::from_slice(&dup_bytes).unwrap();
    assert_eq!(dup_v["error"]["code"], "org.slug_taken");
}

/// Org-owned public repo resolves via org slug (D-ORG-01 / OwnerRef) — not user-only lookup.
#[tokio::test]
async fn org_owned_repo_resolves_by_org_slug() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("org_resolve_slug.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), dir.path().join("repos")).await;

    let (cookie, login_v) = signup_and_login(&app, "resolve@ex.com", "resolve1").await;
    let user_id = login_v["data"]["id"].as_str().expect("id");
    verify_user(&db, user_id).await;

    let create = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"org.create","input":{"slug":"acme-resolve"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let org_id = v["data"]["id"].as_str().expect("org id");

    // Fixture: org-owned row (owner_id = org). owner_type set via insert API once GREEN;
    // until then insert defaults owner_type=user but lookup keys on owner_id.
    db.insert_repository("r-org-resolve", org_id, "widget", "public", "", "main")
        .await
        .expect("insert org-owned repo");

    let get = app
        .oneshot(rpc_req(
            r#"{"procedure":"repo.get","input":{"owner":"acme-resolve","name":"widget"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK, "org slug must resolve like username");
    let get_bytes = get.into_body().collect().await.unwrap().to_bytes();
    let get_v: serde_json::Value = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(get_v["ok"], true, "repo.get under org slug — {get_v}");
    assert_eq!(get_v["data"]["name"], "widget");
    assert_eq!(
        get_v["data"]["owner_username"], "acme-resolve",
        "AccessibleRepo.owner_username is the org slug"
    );
    assert_eq!(get_v["data"]["owner_id"], org_id);
}

/// Creator of an org is Owner (ORG-01 / D-ORG-02a).
#[tokio::test]
async fn org_create_creator_is_owner() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!(
        "sqlite:{}",
        dir.path().join("org_create_owner.db").display()
    );
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), dir.path().join("repos")).await;

    let (cookie, login_v) = signup_and_login(&app, "boss@ex.com", "boss1").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    verify_user(&db, &user_id).await;

    let create = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"org.create","input":{"slug":"boss-org"}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let bytes = create.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let org_id = v["data"]["id"].as_str().expect("org id");

    let member = db
        .find_org_member(org_id, &user_id)
        .await
        .expect("find member")
        .expect("creator membership row");
    assert_eq!(member.role, "owner", "creator must be Owner");
}

/// `members.add` by username adds an existing instance user (ORG-01 / D-ORG-03).
#[tokio::test]
async fn org_members_add_by_username() {
    assert!(
        false,
        "Wave 0: members.add by username must link an existing user (ORG-01 / D-ORG-03)"
    );
}

/// `members.updateRole` changes Owner/Admin/Member (ORG-02 / D-ORG-02a).
#[tokio::test]
async fn org_members_update_role() {
    assert!(
        false,
        "Wave 0: members.updateRole must set Owner|Admin|Member (ORG-02 / D-ORG-02a)"
    );
}

/// Demoting or removing the last Owner → `org.last_owner` (ORG-01).
#[tokio::test]
async fn org_members_last_owner_demote_or_remove_rejected() {
    assert!(
        false,
        "Wave 0: last Owner demote/remove → org.last_owner (ORG-01)"
    );
}

/// Member with member_base=none cannot read private org repo (ORG-02 / D-ORG-02b / T-10-02).
#[tokio::test]
async fn org_member_base_none_denies_private_repo_read() {
    assert!(
        false,
        "Wave 0: Member + member_base none → deny private org repo read (ORG-02 / D-ORG-02b)"
    );
}

/// Member with member_base=read can read private org repo (ORG-02 / D-ORG-02b).
#[tokio::test]
async fn org_member_base_read_allows_private_repo_read() {
    assert!(
        false,
        "Wave 0: Member + member_base read → can read private org repo (ORG-02 / D-ORG-02b)"
    );
}

/// Member with member_base=write can write private org repo (ORG-02 / D-ORG-02b).
#[tokio::test]
async fn org_member_base_write_allows_private_repo_write() {
    assert!(
        false,
        "Wave 0: Member + member_base write → can write private org repo (ORG-02 / D-ORG-02b)"
    );
}
