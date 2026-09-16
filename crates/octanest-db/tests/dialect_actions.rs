//! Phase 19 Wave 0 stub — Actions migration parity (D-ACT-13, D-ACT-17, D-ACT-19).
//! Greened by 19-02. Expect postgres/mysql/sqlite parity for runners, runs, jobs,
//! commit statuses, secrets, and repositories.actions_enabled.

#![allow(dead_code)]

/// Actions domain tables must exist in a dialect migration after 0020_social.
#[tokio::test]
async fn dialect_actions_schema_presence() {
    assert!(
        false,
        "expected actions migration defining runners, runs, jobs, commit_statuses, secrets, actions_enabled (D-ACT-13 / D-ACT-17)"
    );
}

/// Factory reset must wipe Actions runs, log dir, and runner registrations (D-ACT-19).
#[tokio::test]
async fn dialect_actions_factory_reset_scope() {
    assert!(
        false,
        "expected factory reset to wipe actions runs/runners/secrets metadata (D-ACT-19)"
    );
}
