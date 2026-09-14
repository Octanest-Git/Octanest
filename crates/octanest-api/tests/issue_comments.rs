//! ISS-02 Wave 0 stubs: comment CRUD, author edit/delete, Write+ moderate, history.
//!
//! RED until comment RPC lands. Do not implement production handlers here.
//! Covers D-ISS-09, D-ISS-12; Write+ create/comment per D-ISS-20.

/// Write+ can create a comment on an issue (ISS-02 / D-ISS-20).
#[tokio::test]
async fn issue_comments_create() {
    assert!(
        false,
        "Wave 0: comment.create must allow Write+ on an issue (ISS-02 / D-ISS-20)"
    );
}

/// Author can edit own comment body (ISS-02 / D-ISS-09).
#[tokio::test]
async fn issue_comments_author_edit() {
    assert!(
        false,
        "Wave 0: comment author must edit own comment (ISS-02 / D-ISS-09)"
    );
}

/// Author can delete own comment (ISS-02 / D-ISS-09).
#[tokio::test]
async fn issue_comments_author_delete() {
    assert!(
        false,
        "Wave 0: comment author must delete own comment (ISS-02 / D-ISS-09)"
    );
}

/// Write+ can moderate-delete another user's comment (ISS-02 / D-ISS-09 / T-11-02).
#[tokio::test]
async fn issue_comments_write_moderate_delete() {
    assert!(
        false,
        "Wave 0: Write+ must moderate-delete others' comments (ISS-02 / D-ISS-09 / T-11-02)"
    );
}

/// Full edit history on comments (ISS-02 / D-ISS-12).
#[tokio::test]
async fn issue_comments_edit_history_trail() {
    assert!(
        false,
        "Wave 0: comment history must return full body revision trail (ISS-02 / D-ISS-12)"
    );
}
