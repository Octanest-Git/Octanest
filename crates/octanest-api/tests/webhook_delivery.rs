//! Phase 18 Wave 0: HOOK-02/03 delivery + HMAC stubs (D-HOOK-12, D-HOOK-13).
//! Greened in 18-01 / 18-02 / 18-03.

#[tokio::test]
#[ignore = "Wave 0 stub — greened with issues opened delivery (HOOK-02)"]
async fn webhook_issues_deliver_opened() {
    assert!(false, "Wave 0: issue.create delivers signed issues opened POST");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with issues lifecycle actions (D-HOOK-08)"]
async fn webhook_issues_edited_closed_reopened() {
    assert!(false, "Wave 0: issue.update maps to edited/closed/reopened");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with HMAC X-Hub-Signature-256 (D-HOOK-12)"]
async fn webhook_hmac_signature() {
    assert!(false, "Wave 0: X-Hub-Signature-256 matches HMAC-SHA256 of raw body");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with SSRF rejection (D-HOOK-18)"]
async fn webhook_ssrf_rejects_unsafe_url() {
    assert!(false, "Wave 0: SSRF-unsafe URLs rejected on create/delivery");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with delivery timeout handling (D-HOOK-18)"]
async fn webhook_timeout_records_error() {
    assert!(false, "Wave 0: timeout/connection errors recorded without panic");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with retry backoff (D-HOOK-14)"]
async fn webhook_retry_transient() {
    assert!(false, "Wave 0: transient 5xx/timeout retries with attempt rows");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with push HTTPS emit (D-HOOK-10)"]
async fn webhook_push_https_receive() {
    assert!(false, "Wave 0: successful HTTPS receive-pack emits push webhook");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with push SSH emit (D-HOOK-22)"]
async fn webhook_push_ssh_receive() {
    assert!(false, "Wave 0: successful SSH receive emits push webhook");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with pull_request emitters (D-HOOK-09)"]
async fn webhook_pull_request_lifecycle() {
    assert!(false, "Wave 0: PR open/edit/close/reopen/sync/merge emit pull_request");
}
