//! ORG-06 / D-14 / D-19: Direct push denied on protected branches (Wave 0 RED stubs).

mod support;

/// Write non-Admin cannot push directly when required reviews are enabled (ORG-06, D-14).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-02"]
async fn branch_protect_push_denies_direct_push_when_reviews_required() {
    let _ = support::unlock_signup;
    assert!(
        false,
        "TODO 13-02: git push to protected main rejected for Write non-Admin"
    );
}

/// Force-push denied when allow_force_pushes is false (D-15).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-05"]
async fn branch_protect_push_denies_force_push() {
    assert!(
        false,
        "TODO 13-05: non-fast-forward update denied when allow_force_pushes=false"
    );
}

/// lock_branch rejects all pushes for non-bypass actors (D-18).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-05"]
async fn branch_protect_push_lock_branch() {
    assert!(false, "TODO 13-05: lock_branch denies Push intent");
}
