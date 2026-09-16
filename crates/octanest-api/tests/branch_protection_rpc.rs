//! ORG-05 / D-03 / D-26: Admin branch protection rule CRUD (Wave 0 RED stubs).
//!
//! Turned green by plans 13-02 / 13-03 (`repo.branchProtection.*`).

mod support;

/// Admin can create/list/update/delete a rule (ORG-05, D-03, D-26).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-02/13-03"]
async fn branch_protection_rpc_admin_crud() {
    let _ = support::unlock_signup;
    assert!(
        false,
        "TODO 13-03: Admin create/list/update/delete repo.branchProtection rules"
    );
}

/// Non-Admin mutate is soft-denied (Admin gate D-03).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-03"]
async fn branch_protection_rpc_non_admin_denied() {
    assert!(
        false,
        "TODO 13-03: non-Admin branchProtection mutate → soft not_found"
    );
}
