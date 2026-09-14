//! ISS-03 Wave 0 stubs: multi-assignee + Read+ eligibility rejection.
//!
//! RED until assignee RPC lands. Do not implement production handlers here.
//! Covers D-ISS-06, D-ISS-08; Write+ assign per D-ISS-07 / D-ISS-20.

/// Write+ can assign multiple users to an issue (ISS-03 / D-ISS-06).
#[tokio::test]
async fn issue_assignees_multi_assign() {
    assert!(
        false,
        "Wave 0: issue must support multiple assignees (ISS-03 / D-ISS-06)"
    );
}

/// Assignee eligibility = Read+ on the repo; ineligible user rejected (ISS-03 / D-ISS-08).
#[tokio::test]
async fn issue_assignees_reject_without_read_access() {
    assert!(
        false,
        "Wave 0: assign must reject users without Read+ on the repo (ISS-03 / D-ISS-08)"
    );
}

/// Write+ can unassign (ISS-03 / D-ISS-07).
#[tokio::test]
async fn issue_assignees_write_unassign() {
    assert!(
        false,
        "Wave 0: Write+ must unassign users from an issue (ISS-03 / D-ISS-07)"
    );
}
