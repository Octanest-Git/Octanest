//! ISS-04 Wave 0 stubs: link stubs + manual add/remove; no closing-keyword enforcement.
//!
//! RED until link RPC lands. Do not implement production handlers here.
//! Covers D-ISS-13, D-ISS-14, D-ISS-15 (closing keywords deferred to Phase 12).

/// Manual link control writes opaque PR/issue stub rows (ISS-04 / D-ISS-13 / D-ISS-14).
#[tokio::test]
async fn issue_links_manual_add_stub() {
    assert!(
        false,
        "Wave 0: manual Link issue/PR must write stub link rows (ISS-04 / D-ISS-13 / D-ISS-14)"
    );
}

/// Manual remove deletes a stub link (ISS-04 / D-ISS-14).
#[tokio::test]
async fn issue_links_manual_remove() {
    assert!(
        false,
        "Wave 0: manual unlink must remove stub link rows (ISS-04 / D-ISS-14)"
    );
}

/// Closing keywords (`fixes` / `closes` `#N`) are not enforced in Phase 11 (D-ISS-15).
#[tokio::test]
async fn issue_links_no_closing_keyword_enforcement() {
    assert!(
        false,
        "Wave 0: closing keywords must NOT auto-close in Phase 11 (D-ISS-15 → Phase 12)"
    );
}

/// Linked PRs panel lists stub rows until Phase 12 objects exist (D-ISS-13).
#[tokio::test]
async fn issue_links_list_stubs_for_panel() {
    assert!(
        false,
        "Wave 0: issue.links.list must return stub rows for Linked PRs panel (D-ISS-13)"
    );
}
