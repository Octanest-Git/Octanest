//! GIT-11: pat.createClassic / createFineGrained / list / revoke + verified gate.

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::{build_cors, router_with_state, AppState};
use octanest_core::{CLASSIC_PAT_PREFIX, FINE_GRAINED_PAT_PREFIX};
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
    let prefix = v["data"]["item"]["token_prefix"].as_str().expect("token_prefix");
    assert!(
        prefix.starts_with(CLASSIC_PAT_PREFIX) && prefix.len() == CLASSIC_PAT_PREFIX.len() + 8,
        "token_prefix must be brand + 8 hex fingerprint — {prefix}"
    );
    assert!(
        token.starts_with(prefix),
        "plaintext must start with display prefix — {token} / {prefix}"
    );
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
    assert_eq!(items[0]["token_prefix"].as_str().unwrap().len(), CLASSIC_PAT_PREFIX.len() + 8);
    assert!(plaintext.starts_with(items[0]["token_prefix"].as_str().unwrap()));
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

/// Verified `pat.createFineGrained` all + contents write returns one-time `octanest_fg_` token.
#[tokio::test]
async fn pat_create_fine_grained_all_returns_fg_token() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_fg_all.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "fgall@ex.com", "fgalluser").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let (status, v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createFineGrained","input":{"name":"ci-all","repo_access":"all","contents":"write"}}"#,
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "createFineGrained all — {v}");
    assert_eq!(v["ok"], true, "{v}");
    let token = v["data"]["token"].as_str().expect("token");
    assert!(
        token.starts_with(FINE_GRAINED_PAT_PREFIX),
        "must mint {FINE_GRAINED_PAT_PREFIX}* — got {token}"
    );
    assert_eq!(v["data"]["item"]["kind"], "fine_grained");
    assert_eq!(v["data"]["item"]["name"], "ci-all");
    let prefix = v["data"]["item"]["token_prefix"].as_str().expect("token_prefix");
    assert!(
        prefix.starts_with(FINE_GRAINED_PAT_PREFIX)
            && prefix.len() == FINE_GRAINED_PAT_PREFIX.len() + 8,
        "token_prefix must be brand + 8 hex fingerprint — {prefix}"
    );
    assert!(
        token.starts_with(prefix),
        "plaintext must start with display prefix — {token} / {prefix}"
    );
    assert_eq!(v["data"]["item"]["repo_access"], "all");
    assert_eq!(v["data"]["item"]["contents"], "write");
    let repos = v["data"]["item"]["repository_ids"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(repos.is_empty(), "all mode must not persist join rows — {v}");
    assert!(v["data"]["item"].get("token").is_none());
}

/// Selected FG with owned repo ids persists join rows; list shows fine_grained kind.
#[tokio::test]
async fn pat_create_fine_grained_selected_persists_repos() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_fg_sel.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "fgsel@ex.com", "fgseluser").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let repo = db
        .insert_repository("r-fg-1", &user_id, "user", "demo", "public", "", "main")
        .await
        .expect("insert repo");

    let (status, v) = rpc_json(
        &app,
        &format!(
            r#"{{"procedure":"pat.createFineGrained","input":{{"name":"laptop-fg","repo_access":"selected","contents":"read","repository_ids":["{}"]}}}}"#,
            repo.id
        ),
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "createFineGrained selected — {v}");
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["data"]["item"]["kind"], "fine_grained");
    assert_eq!(v["data"]["item"]["repo_access"], "selected");
    assert_eq!(v["data"]["item"]["contents"], "read");
    let ids = v["data"]["item"]["repository_ids"]
        .as_array()
        .expect("repository_ids");
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], repo.id);

    let (list_status, list_v) =
        rpc_json(&app, r#"{"procedure":"pat.list","input":{}}"#, &cookie).await;
    assert_eq!(list_status, StatusCode::OK, "{list_v}");
    let items = list_v["data"].as_array().expect("list");
    let fg = items
        .iter()
        .find(|i| i["kind"] == "fine_grained")
        .expect("fine_grained in list");
    let fg_prefix = fg["token_prefix"].as_str().expect("token_prefix");
    assert!(
        fg_prefix.starts_with(FINE_GRAINED_PAT_PREFIX)
            && fg_prefix.len() == FINE_GRAINED_PAT_PREFIX.len() + 8,
        "token_prefix must be brand + 8 hex fingerprint — {fg_prefix}"
    );
    assert_eq!(fg["repository_ids"].as_array().unwrap()[0], repo.id);
}

/// Selected with empty repository_ids → pat.repos_required.
#[tokio::test]
async fn pat_create_fine_grained_selected_empty_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_fg_empty.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie, login_v) = signup_and_login(&app, "fgempty@ex.com", "fgemptyu").await;
    let user_id = login_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_id, &now)
        .await
        .expect("verify");

    let (status, v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createFineGrained","input":{"name":"bad","repo_access":"selected","contents":"write","repository_ids":[]}}"#,
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{v}");
    assert_eq!(v["error"]["code"], "pat.repos_required");
}

/// Unverified createFineGrained → auth.email_unverified (D-24).
#[tokio::test]
async fn pat_create_fine_grained_unverified_email_unverified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_fg_unv.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db).await;

    let (cookie, login_v) = signup_and_login(&app, "fgnew@ex.com", "fgnewbie").await;
    assert_eq!(login_v["data"]["email_verified"], false);

    let (status, v) = rpc_json(
        &app,
        r#"{"procedure":"pat.createFineGrained","input":{"name":"x","repo_access":"all","contents":"read"}}"#,
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{v}");
    assert_eq!(v["error"]["code"], "auth.email_unverified");
}

/// Selected with a repo not owned by caller → pat.invalid_scope (T-08-06).
#[tokio::test]
async fn pat_create_fine_grained_foreign_repo_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_fg_foreign.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (cookie_a, login_a) = signup_and_login(&app, "fga@ex.com", "fgauser").await;
    let user_a = login_a["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&user_a, &now)
        .await
        .expect("verify a");

    let (cookie_b, login_b) = signup_and_login(&app, "fgb@ex.com", "fgbuser").await;
    let user_b = login_b["data"]["id"].as_str().expect("id").to_string();
    db.set_email_verified_at(&user_b, &now)
        .await
        .expect("verify b");

    let foreign = db
        .insert_repository("r-foreign", &user_a, "user", "secrets", "private", "", "main")
        .await
        .expect("foreign repo");

    let (status, v) = rpc_json(
        &app,
        &format!(
            r#"{{"procedure":"pat.createFineGrained","input":{{"name":"steal","repo_access":"selected","contents":"write","repository_ids":["{}"]}}}}"#,
            foreign.id
        ),
        &cookie_b,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{v}");
    assert_eq!(v["error"]["code"], "pat.invalid_scope");
    let _ = cookie_a; // keep a session created for ownership fixture
}

/// ORG-04 / A4: FG Selected may include repos where subject has ACL capability.
#[tokio::test]
async fn pat_create_fine_grained_selected_allows_collaborator_repo() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_fg_collab.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (_owner_cookie, owner_v) = signup_and_login(&app, "fgco@ex.com", "fgcoown").await;
    let owner_id = owner_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&owner_id, &now)
        .await
        .expect("verify owner");

    let (collab_cookie, collab_v) = signup_and_login(&app, "fgcc@ex.com", "fgccoll").await;
    let collab_id = collab_v["data"]["id"].as_str().expect("id").to_string();
    db.set_email_verified_at(&collab_id, &now)
        .await
        .expect("verify collab");

    let repo = db
        .insert_repository("r-collab-fg", &owner_id, "user", "shared", "private", "", "main")
        .await
        .expect("repo");
    db.insert_repo_collaborator(&repo.id, &collab_id, "write")
        .await
        .expect("grant write");

    let (status, v) = rpc_json(
        &app,
        &format!(
            r#"{{"procedure":"pat.createFineGrained","input":{{"name":"collab-fg","repo_access":"selected","contents":"write","repository_ids":["{}"]}}}}"#,
            repo.id
        ),
        &collab_cookie,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "collaborator Write must mint Selected FG — {v}"
    );
    assert_eq!(v["ok"], true, "{v}");
    let ids = v["data"]["item"]["repository_ids"]
        .as_array()
        .expect("repository_ids");
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], repo.id);
}

/// Read collaborator cannot mint Selected FG with contents:write (capability mismatch).
#[tokio::test]
async fn pat_create_fine_grained_selected_read_collab_write_contents_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("pat_fg_read_deny.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db.clone()).await;

    let (_owner_cookie, owner_v) = signup_and_login(&app, "fgrd@ex.com", "fgrdown").await;
    let owner_id = owner_v["data"]["id"].as_str().expect("id").to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(&owner_id, &now)
        .await
        .expect("verify owner");

    let (collab_cookie, collab_v) = signup_and_login(&app, "fgrc@ex.com", "fgrcoll").await;
    let collab_id = collab_v["data"]["id"].as_str().expect("id").to_string();
    db.set_email_verified_at(&collab_id, &now)
        .await
        .expect("verify collab");

    let repo = db
        .insert_repository("r-read-fg", &owner_id, "user", "ro", "private", "", "main")
        .await
        .expect("repo");
    db.insert_repo_collaborator(&repo.id, &collab_id, "read")
        .await
        .expect("grant read");

    let (status, v) = rpc_json(
        &app,
        &format!(
            r#"{{"procedure":"pat.createFineGrained","input":{{"name":"too-much","repo_access":"selected","contents":"write","repository_ids":["{}"]}}}}"#,
            repo.id
        ),
        &collab_cookie,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{v}");
    assert_eq!(v["error"]["code"], "pat.invalid_scope");
}
