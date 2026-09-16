//! PR-08 / D-22: PR merge blocked until protection satisfied (Wave 0 RED stubs).

mod support;

/// Merge into protected base fails without required approvals (PR-08, D-22).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-02"]
async fn branch_protect_merge_blocked_without_approval() {
    let _ = support::unlock_signup;
    assert!(
        false,
        "TODO 13-02: pull.merge returns structured protection reasons when Approves missing"
    );
}

/// After eligible Approve, merge succeeds (PR-08).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-02"]
async fn branch_protect_merge_succeeds_after_approval() {
    assert!(
        false,
        "TODO 13-02: pull.merge succeeds after Write+ Approve from non-author"
    );
}

/// Required status context blocks merge until success (D-10).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-04"]
async fn branch_protect_merge_requires_status_context() {
    assert!(
        false,
        "TODO 13-04: merge blocked until required context is success on head sha"
    );
}
