//! Phase 18 Wave 0: HOOK-01 Admin webhook CRUD RPC stubs (D-HOOK-02).
//! Greened in 18-01 / 18-02.

#[tokio::test]
#[ignore = "Wave 0 stub — greened with webhook.create (HOOK-01)"]
async fn webhook_create_admin() {
    assert!(false, "Wave 0: Admin webhook.create returns hook + one-time secret");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with webhook.list (HOOK-01)"]
async fn webhook_list_admin() {
    assert!(false, "Wave 0: Admin webhook.list returns repo hooks with masked secrets");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with webhook.update (HOOK-01)"]
async fn webhook_update_admin() {
    assert!(false, "Wave 0: Admin webhook.update mutates URL/events/active/secret");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with webhook.delete (HOOK-01)"]
async fn webhook_delete_admin() {
    assert!(false, "Wave 0: Admin webhook.delete removes hook");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with Admin gate denial (D-HOOK-02)"]
async fn webhook_admin_denial() {
    assert!(false, "Wave 0: non-Admin webhook.* soft not_found / denied");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with inactive skip (D-HOOK-15)"]
async fn webhook_inactive_skips_enqueue() {
    assert!(false, "Wave 0: inactive webhook skips delivery enqueue");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with deliveries.list (HOOK-03)"]
async fn webhook_deliveries_list() {
    assert!(false, "Wave 0: Admin webhook.deliveries.list returns recent status");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with webhook.ping (D-HOOK-21)"]
async fn webhook_ping() {
    assert!(false, "Wave 0: Admin webhook.ping enqueues ping delivery");
}

#[tokio::test]
#[ignore = "Wave 0 stub — greened with webhook.redeliver (D-HOOK-21)"]
async fn webhook_redeliver() {
    assert!(false, "Wave 0: Admin webhook.redeliver enqueues new delivery");
}
