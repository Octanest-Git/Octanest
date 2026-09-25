//! admin.auth.* requires sys-admin (T-04-21).

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oxidean_api::auth::hash_password_str;
use oxidean_api::email::{EmailSender, LogSink};
use oxidean_api::{build_cors, router_with_state, AppState};
use oxidean_db::Database;
use tower::ServiceExt;
use uuid::Uuid;

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

fn session_cookie_from_response(res: &axum::http::Response<Body>) -> String {
    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie")
        .to_str()
        .unwrap();
    set_cookie.split(';').next().unwrap().trim().to_string()
}

#[tokio::test]
async fn non_admin_get_settings_forbidden() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("admin_forbid.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    support::unlock_signup(&db).await;
    let app = test_app(db).await;

    let signup = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.signup","input":{"email":"user@ex.com","username":"normie","password":"password1"}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&signup);

    let res = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.auth.get_settings","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "admin.forbidden");
}

#[tokio::test]
async fn admin_get_and_update_settings() {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("sqlite:{}", dir.path().join("admin_ok.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");

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
        oxidean_core::Role::SysAdmin,
    )
    .await
    .expect("create admin");

    let app = test_app(db).await;
    let login = app
        .clone()
        .oneshot(rpc_req(
            r#"{"procedure":"auth.login","input":{"identifier":"admin@ex.com","password":"password1","remember_me":false}}"#,
        ))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&login);

    let get = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.auth.get_settings","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK);
    let get_bytes = get.into_body().collect().await.unwrap().to_bytes();
    let get_v: serde_json::Value = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(get_v["ok"], true);
    assert!(get_v["data"]["smtp_configured"].is_boolean());
    assert!(get_v["data"]["resend_configured"].is_boolean());
    assert!(get_v["data"]["workos_api_key_configured"].is_boolean());
    assert_eq!(
        get_v["data"]["allow_signup"], false,
        "D-07: get_settings must return allow_signup (default false)"
    );
    // No secret values in payload
    let raw = get_bytes.to_vec();
    let s = String::from_utf8_lossy(&raw);
    assert!(!s.contains("OXIDEAN_RESEND_API_KEY"));
    assert!(!s.contains("WORKOS_API_KEY"));

    let update = app
        .clone()
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.auth.update_settings","input":{"provider_mode":"local","email_provider":"log","from_address":"Oxidean <noreply@test>","oidc_issuer":null,"oidc_client_id":null,"workos_client_id":"wk_display","allow_signup":true}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);
    let update_bytes = update.into_body().collect().await.unwrap().to_bytes();
    let update_v: serde_json::Value = serde_json::from_slice(&update_bytes).unwrap();
    assert_eq!(update_v["data"]["workos_client_id"], "wk_display");
    assert_eq!(update_v["data"]["from_address"], "Oxidean <noreply@test>");
    assert_eq!(
        update_v["data"]["allow_signup"], true,
        "D-08: update_settings must persist allow_signup for sys-admin"
    );

    let get2 = app
        .oneshot(rpc_req_with_cookie(
            r#"{"procedure":"admin.auth.get_settings","input":{}}"#,
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(get2.status(), StatusCode::OK);
    let get2_bytes = get2.into_body().collect().await.unwrap().to_bytes();
    let get2_v: serde_json::Value = serde_json::from_slice(&get2_bytes).unwrap();
    assert_eq!(
        get2_v["data"]["allow_signup"], true,
        "allow_signup must round-trip through get_settings after update"
    );
}
