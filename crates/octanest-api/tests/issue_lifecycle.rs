//! ISS-01 Wave 0 stubs: create/edit/close/reopen + per-repo #N + history.
//!
//! RED until issue RPC lands (11-02+). Do not implement production handlers here.
//! Covers D-ISS-01..04; ACL capability matrix in D-ISS-20.

/// Verified Write+ can create an issue and receives per-repo `#1` (ISS-01 / D-ISS-01).
#[tokio::test]
async fn issue_lifecycle_create_allocates_per_repo_number() {
    assert!(
        false,
        "Wave 0: issue.create must allocate monotonic per-repo #N via issue_counters (ISS-01 / D-ISS-01)"
    );
}

/// Author or Write+ may edit title/body after create (ISS-01 / D-ISS-03).
#[tokio::test]
async fn issue_lifecycle_edit_title_body() {
    assert!(
        false,
        "Wave 0: issue.edit must allow author + Write+ to change title/body (ISS-01 / D-ISS-03)"
    );
}

/// Lifecycle is open ↔ closed; reopen allowed (ISS-01 / D-ISS-02).
#[tokio::test]
async fn issue_lifecycle_close_and_reopen() {
    assert!(
        false,
        "Wave 0: issue.close / issue.reopen must toggle open↔closed (ISS-01 / D-ISS-02)"
    );
}

/// Full edit history trail for title/body (ISS-01 / D-ISS-04).
#[tokio::test]
async fn issue_history_full_title_body_trail() {
    assert!(
        false,
        "Wave 0: issue.history must return full title/body revision trail (ISS-01 / D-ISS-04)"
    );
}

/// Second create in same repo gets `#2` (monotonic, no gaps from open edits) (D-ISS-01).
#[tokio::test]
async fn issue_lifecycle_second_create_monotonic_number() {
    assert!(
        false,
        "Wave 0: second issue.create in repo must allocate #2 (ISS-01 / D-ISS-01)"
    );
}
