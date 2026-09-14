//! ISS-01 Wave 0 stubs: Admin hard-delete with confirmNumber; no #N reuse.
//!
//! RED until issue delete RPC lands. Do not implement production handlers here.
//! Threat: T-11-02 — Admin-only hard-delete (D-ISS-02 / D-ISS-20).

/// Repo Admin hard-deletes with typed `confirmNumber` matching `#N` (D-ISS-02).
#[tokio::test]
async fn issue_delete_admin_hard_delete_requires_confirm_number() {
    assert!(
        false,
        "Wave 0: Admin hard-delete must require confirmNumber matching issue #N (D-ISS-02 / T-11-02)"
    );
}

/// Non-Admin Write cannot hard-delete (D-ISS-20 / T-11-02).
#[tokio::test]
async fn issue_delete_write_role_rejected() {
    assert!(
        false,
        "Wave 0: Write without Admin must not hard-delete issues (D-ISS-20 / T-11-02)"
    );
}

/// After hard-delete, `#N` is never reused; next create continues counters (D-ISS-01 / D-ISS-02).
#[tokio::test]
async fn issue_delete_number_not_reused_after_hard_delete() {
    assert!(
        false,
        "Wave 0: hard-delete must not reclaim #N; issue_counters keep max_number (D-ISS-01 / D-ISS-02)"
    );
}
