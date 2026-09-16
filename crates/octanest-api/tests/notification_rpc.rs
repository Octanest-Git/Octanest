//! Phase 17 Wave 0 stubs — NOTF-01 / NOTF-02 notification RPC behaviors.
//!
//! Greened by 17-01+ (list/unread/mark + comment→author; later issue/PR emitters).

#[tokio::test]
#[ignore = "Wave 0 stub — greened with notification.list empty for signed-in user"]
async fn notification_list_empty_when_signed_in() {
    assert!(false, "Wave 0: signed-in notification.list returns empty initially");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with issue.comments.create → author unread (NOTF-01 / D-01 / D-03)"]
async fn notification_issue_comment_creates_unread_for_author() {
    assert!(
        false,
        "Wave 0: B comments on A's issue → A unreadCount=1; reason issue_comment; subject_kind issue"
    );
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with notification.unreadCount"]
async fn notification_unread_count() {
    assert!(false, "Wave 0: notification.unreadCount reflects unread rows for session user");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with notification.markRead + markAllRead (NOTF-02 / D-12)"]
async fn notification_mark_read_and_mark_all_read() {
    assert!(false, "Wave 0: markRead clears one; markAllRead clears remaining");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened when actor is never notified (D-03)"]
async fn notification_actor_is_not_notified() {
    assert!(false, "Wave 0: comment author does not receive a notification for own comment");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with markRead IDOR guard (T-17-01)"]
async fn notification_cannot_mark_another_users_notification() {
    assert!(
        false,
        "Wave 0: markRead with another user's notification id does not mutate that row"
    );
}
