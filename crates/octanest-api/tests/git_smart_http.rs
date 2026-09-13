//! GIT-02 Wave 0 stubs: Git Smart HTTP auth / ACL / status codes.
//!
//! RED until Smart HTTP routes land (08-04 / 08-06). Do not implement handlers here.
//! Threat mitigations: T-08-02 — password reject, cookie ignore, private 401 (D-11, D-12, D-21).

/// Public repo: anonymous `info/refs?service=git-upload-pack` allowed.
#[tokio::test]
async fn git_smart_public_anon_upload_pack_info_refs_ok() {
    assert!(
        false,
        "Wave 0: public anon upload-pack info/refs must succeed (GIT-02)"
    );
}

/// Private repo: anonymous → 401 + WWW-Authenticate (D-21).
#[tokio::test]
async fn git_smart_private_anon_401_www_authenticate() {
    assert!(
        false,
        "Wave 0: private anon → 401 + WWW-Authenticate Basic (GIT-02 / D-21)"
    );
}

/// Basic auth with account password (not PAT) → 401 + PAT hint (D-11).
#[tokio::test]
async fn git_smart_basic_account_password_rejected_401() {
    assert!(
        false,
        "Wave 0: account password over Basic → 401 + PAT hint; never accept password (GIT-02 / D-11)"
    );
}

/// Session cookie alone is treated as anonymous (D-12) — never authenticates Smart HTTP.
#[tokio::test]
async fn git_smart_session_cookie_ignored_as_anon() {
    assert!(
        false,
        "Wave 0: session cookie alone treated as anon on Smart HTTP (GIT-02 / D-12)"
    );
}

/// Valid PAT with insufficient scope → 403.
#[tokio::test]
async fn git_smart_insufficient_scope_403() {
    assert!(
        false,
        "Wave 0: insufficient PAT scope → 403 (GIT-02)"
    );
}

/// Failed-auth over limit → 429 + Retry-After (D-26).
#[tokio::test]
async fn git_smart_failed_auth_rate_limit_429_retry_after() {
    assert!(
        false,
        "Wave 0: failed-auth over limit → 429 + Retry-After (GIT-02 / D-26)"
    );
}

/// Unverified owner push (receive-pack) denied (D-24).
#[tokio::test]
async fn git_smart_unverified_push_denied() {
    assert!(
        false,
        "Wave 0: unverified email cannot push over Smart HTTP (GIT-02 / D-24)"
    );
}

/// PAT push/fetch happy-path stub (GIT-02 end-to-end once handlers land).
#[tokio::test]
async fn git_smart_pat_push_fetch_happy_path() {
    assert!(
        false,
        "Wave 0: valid PAT must allow fetch + push on owned repo (GIT-02)"
    );
}
