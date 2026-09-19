//! ORG-06 / D-14 / D-19: Direct push denied on protected branches.

mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::email::{EmailSender, LogSink};
use octanest_api::git::bare_repo_path;
use octanest_api::protection::{
    check_ref_update, hooks_installed, reconcile_hooks, ProtectionIntent, ZERO_SHA,
};
use octanest_api::repo::Capability;
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
    res.headers()
        .get("set-cookie")
        .expect("Set-Cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .trim()
        .to_string()
}

async fn signup_and_login(
    app: &axum::Router,
    email: &str,
    username: &str,
) -> (String, serde_json::Value) {
    let signup = app
        .clone()
        .oneshot(rpc_req(&format!(
            r#"{{"procedure":"auth.signup","input":{{"email":"{email}","username":"{username}","password":"password1"}}}}"#
        )))
        .await
        .unwrap();
    assert_eq!(signup.status(), StatusCode::OK);
    let _ = signup.into_body().collect().await;
    let login = app
        .clone()
        .oneshot(rpc_req(&format!(
            r#"{{"procedure":"auth.login","input":{{"identifier":"{email}","password":"password1","remember_me":false}}}}"#
        )))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&login);
    let bytes = login.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (cookie, v)
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

async fn verify_user(db: &Database, user_id: &str) {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    db.set_email_verified_at(user_id, &now).await.expect("verify");
}

/// Write non-Admin cannot push directly when required reviews are enabled (ORG-06, D-14).
#[tokio::test]
async fn branch_protect_push_denies_direct_push_when_reviews_required() {
    let dir = tempfile::tempdir().unwrap();
    let repos = dir.path().join("repos");
    let db_path = dir.path().join("bp_push.db");
    let url = format!("sqlite:{}", db_path.display());
    let db = Database::connect(&url).await.unwrap();
    db.migrate().await.unwrap();
    support::unlock_signup(&db).await;
    let app = test_app(db.clone(), repos.clone()).await;

    let (owner_cookie, owner_login) =
        signup_and_login(&app, "owner@ex.com", "bpown").await;
    verify_user(&db, owner_login["data"]["id"].as_str().unwrap()).await;
    let create = rpc_json(
        &app,
        &owner_cookie,
        r#"{"procedure":"repo.create","input":{"name":"core","visibility":"public","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}"#,
    )
    .await;
    assert_eq!(create["ok"], true, "{create}");

    let bare = bare_repo_path(&repos, "bpown", "core").expect("bare path");
    assert!(
        hooks_installed(&bare).await,
        "init_bare must install protection hooks (D-19)"
    );

    let rule = rpc_json(
        &app,
        &owner_cookie,
        r#"{"procedure":"repo.branchProtection.create","input":{"owner":"bpown","name":"core","pattern":"main","require_reviews":true,"required_approving_review_count":1}}"#,
    )
    .await;
    assert_eq!(rule["ok"], true, "create rule — {rule}");

    // Simulate Write collaborator push via shared evaluator (same path as hook helper).
    let err = check_ref_update(
        &db,
        &repos,
        &bare,
        "refs/heads/main",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        Capability::Write,
    )
    .await
    .expect_err("Write push must be denied when reviews required");
    assert_eq!(err.code, "repo.branch_protection");
    let reasons = err.data.as_ref().unwrap()["reasons"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        reasons.iter().any(|r| r.as_str() == Some("reviews")),
        "expected reviews reason — {err:?}"
    );

    // Admin bypass when enforce_admins=false (default).
    check_ref_update(
        &db,
        &repos,
        &bare,
        "refs/heads/main",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        Capability::Admin,
    )
    .await
    .expect("Admin may bypass when enforce_admins=false");
}

/// Force-push denied when allow_force_pushes is false (D-15) — greened in 13-05; keep ignored until then if needed.
#[tokio::test]
async fn branch_protect_push_denies_force_push() {
    use octanest_api::protection::{evaluate_push, union_rules};
    use octanest_api::repo::Capability;
    use octanest_db::BranchProtectionRuleRow;

    let rule = BranchProtectionRuleRow {
        id: "1".into(),
        repo_id: "r".into(),
        pattern: "main".into(),
        require_reviews: false,
        required_approving_review_count: 1,
        dismiss_stale_reviews: false,
        require_conversation_resolution: false,
        require_last_push_approval: false,
        required_status_contexts: "[]".into(),
        strict_status_checks: false,
        allow_force_pushes: false,
        allow_deletions: false,
        enforce_admins: false,
        required_linear_history: false,
        lock_branch: false,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let eff = union_rules(&[rule], "main");
    let err = evaluate_push(&eff, ProtectionIntent::ForcePush, Some(Capability::Write))
        .expect_err("force push denied");
    assert_eq!(err.code, "repo.branch_protection");
}

/// lock_branch rejects all pushes for non-bypass actors (D-18).
#[tokio::test]
async fn branch_protect_push_lock_branch() {
    use octanest_api::protection::{evaluate_push, union_rules};
    use octanest_api::repo::Capability;
    use octanest_db::BranchProtectionRuleRow;

    let rule = BranchProtectionRuleRow {
        id: "1".into(),
        repo_id: "r".into(),
        pattern: "main".into(),
        require_reviews: false,
        required_approving_review_count: 1,
        dismiss_stale_reviews: false,
        require_conversation_resolution: false,
        require_last_push_approval: false,
        required_status_contexts: "[]".into(),
        strict_status_checks: false,
        allow_force_pushes: false,
        allow_deletions: false,
        enforce_admins: false,
        required_linear_history: false,
        lock_branch: true,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let eff = union_rules(&[rule], "main");
    let err = evaluate_push(&eff, ProtectionIntent::Push, Some(Capability::Write))
        .expect_err("lock_branch denies push");
    assert_eq!(err.code, "repo.branch_protection");
}

/// Reconcile installs missing hooks (D-19).
#[tokio::test]
async fn branch_protect_push_reconcile_hooks() {
    let dir = tempfile::tempdir().unwrap();
    let bare = dir.path().join("orphan.git");
    std::fs::create_dir_all(&bare).unwrap();
    // Minimal bare layout without hooks.
    std::fs::create_dir_all(bare.join("refs/heads")).unwrap();
    assert!(!hooks_installed(&bare).await);
    reconcile_hooks(&bare).await.expect("reconcile");
    assert!(hooks_installed(&bare).await);
    let _ = ZERO_SHA;
}

/// D-PKG-01: env helper path wins; empty/unset falls back to default_helper_path input.
#[test]
fn branch_protect_default_helper_resolution_prefers_env() {
    use octanest_api::protection::resolve_protection_helper_with;
    use std::path::PathBuf;

    let preferred = resolve_protection_helper_with(
        Some("/from/env/octanest-protection-hook".into()),
        Some(PathBuf::from("/from/sibling/octanest-protection-hook")),
    );
    assert_eq!(
        preferred.as_deref(),
        Some("/from/env/octanest-protection-hook")
    );

    let fallback = resolve_protection_helper_with(
        None,
        Some(PathBuf::from("/from/sibling/octanest-protection-hook")),
    );
    assert_eq!(
        fallback.as_deref(),
        Some("/from/sibling/octanest-protection-hook")
    );
}

/// Invoke installed hooks/update with ref args under a given OCTANEST_ENV / helper.
async fn run_protection_hook_script(
    bare: &std::path::Path,
    octanest_env: Option<&str>,
    helper: Option<&std::path::Path>,
) -> std::process::Output {
    let update = bare.join("hooks").join("update");
    let mut cmd = tokio::process::Command::new(&update);
    cmd.args([
        "refs/heads/main",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ])
    .env_remove("OCTANEST_PROTECTION_HELPER");
    match octanest_env {
        Some(v) => {
            cmd.env("OCTANEST_ENV", v);
        }
        None => {
            cmd.env_remove("OCTANEST_ENV");
        }
    }
    if let Some(h) = helper {
        cmd.env("OCTANEST_PROTECTION_HELPER", h);
    }
    cmd.output().await.expect("spawn hooks/update")
}

/// D-PKG-02: missing helper + production|cloud → fail-closed (non-zero).
#[tokio::test]
async fn protection_hook_script_fail_closed_when_helper_missing_in_production() {
    let dir = tempfile::tempdir().unwrap();
    let bare = dir.path().join("prod.git");
    std::fs::create_dir_all(&bare).unwrap();
    octanest_git::install_protection_hooks(&bare)
        .await
        .expect("install hooks");

    for env_name in ["production", "cloud"] {
        let out = run_protection_hook_script(&bare, Some(env_name), None).await;
        assert!(
            !out.status.success(),
            "OCTANEST_ENV={env_name} must deny when helper missing — stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("protection helper") || stderr.contains("OCTANEST"),
            "stderr should explain missing helper — {stderr}"
        );
    }
}

/// D-PKG-02: missing helper + compose|development|dev (or default) → fail-open.
#[tokio::test]
async fn protection_hook_script_fail_open_when_helper_missing_in_dev() {
    let dir = tempfile::tempdir().unwrap();
    let bare = dir.path().join("dev.git");
    std::fs::create_dir_all(&bare).unwrap();
    octanest_git::install_protection_hooks(&bare)
        .await
        .expect("install hooks");

    for env_name in [Some("compose"), Some("development"), Some("dev"), None] {
        let out = run_protection_hook_script(&bare, env_name, None).await;
        assert!(
            out.status.success(),
            "OCTANEST_ENV={env_name:?} must fail-open when helper missing — stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// D-PKG-02: executable helper is exec'd with ref args.
#[tokio::test]
async fn protection_hook_script_execs_helper_when_present() {
    let dir = tempfile::tempdir().unwrap();
    let bare = dir.path().join("helper.git");
    std::fs::create_dir_all(&bare).unwrap();
    octanest_git::install_protection_hooks(&bare)
        .await
        .expect("install hooks");

    let marker = dir.path().join("helper-ran");
    let helper = dir.path().join("fake-helper.sh");
    let script = format!(
        "#!/bin/sh\necho \"$1 $2\" > {}\nexit 0\n",
        marker.display()
    );
    std::fs::write(&helper, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&helper).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&helper, perms).unwrap();
    }

    let out = run_protection_hook_script(&bare, Some("production"), Some(&helper)).await;
    assert!(
        out.status.success(),
        "helper exec must succeed — stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let ran = std::fs::read_to_string(&marker).expect("helper marker");
    assert!(
        ran.contains("update refs/heads/main"),
        "helper must receive update + refname — {ran}"
    );
}
