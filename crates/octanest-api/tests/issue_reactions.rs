//! D-ISS-11 Wave 0 stubs: GitHub eight reaction content values on issue and comment.
//!
//! RED until reaction RPC lands. Do not implement production handlers here.
//! Write+ may react (D-ISS-20).

/// GitHub eight content values toggle on an issue (D-ISS-11).
#[tokio::test]
async fn issue_reactions_eight_content_on_issue() {
    assert!(
        false,
        "Wave 0: issue reactions must support GitHub eight content values (D-ISS-11)"
    );
}

/// Same eight content values toggle on a comment (D-ISS-11).
#[tokio::test]
async fn issue_reactions_eight_content_on_comment() {
    assert!(
        false,
        "Wave 0: comment reactions must support GitHub eight content values (D-ISS-11)"
    );
}

/// Toggle off removes the caller's reaction (D-ISS-11).
#[tokio::test]
async fn issue_reactions_toggle_off() {
    assert!(
        false,
        "Wave 0: reacting again with same content must remove the reaction (D-ISS-11)"
    );
}
