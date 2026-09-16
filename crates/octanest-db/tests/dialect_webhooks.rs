//! Phase 18 Wave 0: webhooks migration parity stub (HOOK-01..03 schema).
//! Greened in 18-01 when `00NN_webhooks` lands across sqlite/postgres/mysql.

#[tokio::test]
#[ignore = "Wave 0 stub — greened with 00NN_webhooks tri-dialect migration"]
async fn dialect_webhooks_migrate_schema_presence() {
    assert!(
        false,
        "Wave 0: expect webhooks + webhook_deliveries (+ attempts) migration across dialects"
    );
}
