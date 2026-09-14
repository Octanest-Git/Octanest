//! ISS-03 Wave 0 stubs: org+repo label defs Admin-only; Write+ assign.
//!
//! RED until label RPC lands. Do not implement production handlers here.
//! Threat: T-11-02 — Admin for defs; Write+ for assign (D-ISS-05, D-ISS-07, D-ISS-20).

/// Admin can create org-default and repo-local label definitions (ISS-03 / D-ISS-05 / D-ISS-07).
#[tokio::test]
async fn issue_labels_admin_create_org_and_repo_defs() {
    assert!(
        false,
        "Wave 0: Admin-only label definition CRUD for org defaults + repo overrides (ISS-03 / D-ISS-05 / D-ISS-07)"
    );
}

/// Non-Admin Write cannot create/edit/delete label definitions (D-ISS-07 / T-11-02).
#[tokio::test]
async fn issue_labels_write_cannot_mutate_defs() {
    assert!(
        false,
        "Wave 0: Write without Admin must not mutate label definitions (D-ISS-07 / T-11-02)"
    );
}

/// Write+ can assign/unassign labels on an issue (ISS-03 / D-ISS-07).
#[tokio::test]
async fn issue_labels_write_assign_on_issue() {
    assert!(
        false,
        "Wave 0: Write+ must assign/unassign labels on an issue (ISS-03 / D-ISS-07)"
    );
}

/// Effective label set merges org defaults with repo hide/local-only overrides (D-ISS-05).
#[tokio::test]
async fn issue_labels_effective_set_org_plus_repo_overrides() {
    assert!(
        false,
        "Wave 0: effective labels = org defaults ∪ repo overrides (hide/local-only) (D-ISS-05)"
    );
}
