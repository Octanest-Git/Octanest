//! GIT-11 Wave 0 stubs: PAT createClassic / list / revoke + verified gate.
//!
//! RED until pat.* RPC handlers land (08-04+). Do not implement production handlers here.
//! Threat mitigations encoded in assertions: T-08-01 (no plaintext in list; create returns token once).

/// Verified `pat.createClassic` returns a one-time plaintext `token` field (D-15).
#[tokio::test]
async fn pat_create_classic_returns_one_time_token() {
    assert!(
        false,
        "Wave 0: verified createClassic must return ok + data.token once (GIT-11 / D-15)"
    );
}

/// `pat.list` never returns plaintext token secrets (D-15 / T-08-01).
#[tokio::test]
async fn pat_list_omits_secret_token() {
    assert!(
        false,
        "Wave 0: pat.list must omit token secret; only prefix/metadata (GIT-11 / D-15)"
    );
}

/// `pat.revoke` removes the token from subsequent list results.
#[tokio::test]
async fn pat_revoke_removes_from_list() {
    assert!(
        false,
        "Wave 0: after pat.revoke, token id must be absent from pat.list (GIT-11)"
    );
}

/// Unverified session cannot create PATs → `auth.email_unverified` (D-24).
#[tokio::test]
async fn pat_create_unverified_email_unverified() {
    assert!(
        false,
        "Wave 0: unverified createClassic → auth.email_unverified (GIT-11 / D-24)"
    );
}

/// Empty / whitespace note on create → `pat.note_required` (D-16).
#[tokio::test]
async fn pat_create_empty_note_required() {
    assert!(
        false,
        "Wave 0: empty note on createClassic → pat.note_required (GIT-11 / D-16)"
    );
}
