//! Wave 0 / Phase 11: factory_reset_instance must wipe issue domain tables.
//!
//! RED until 0011_issues + cascade/reset order lands. Mirror factory_reset_orgs.
//! Prefer wipe via repository ON DELETE CASCADE (or explicit reset order).

/// After factory_reset_instance, issue domain tables are empty (cascade or ordered delete).
#[tokio::test]
async fn factory_reset_issues_wipes_issue_domain_tables() {
    assert!(
        false,
        "Wave 0: factory_reset_instance must wipe issues/counters/comments/revisions/labels/assignees/reactions/links via repository cascade (or explicit order)"
    );
}

/// Seeding issue rows then deleting the parent repository cascades issue children.
#[tokio::test]
async fn factory_reset_issues_repository_cascade() {
    assert!(
        false,
        "Wave 0: DELETE FROM repositories must cascade wipe issue domain child rows"
    );
}
