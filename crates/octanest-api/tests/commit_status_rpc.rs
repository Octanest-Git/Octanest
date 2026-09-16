//! D-11 / D-12 / D-13: Classic commit status create/list (Wave 0 RED stubs).

mod support;

/// Write+ upserts status by (sha, context); list returns latest-wins (D-11..13).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-04"]
async fn commit_status_rpc_create_list_latest_wins() {
    let _ = support::unlock_signup;
    assert!(
        false,
        "TODO 13-04: repo.commitStatus.create/list latest-wins per context"
    );
}

/// Read+ can list; anonymous/insufficient cannot create (D-13).
#[tokio::test]
#[ignore = "Wave 0 RED — implemented in 13-04"]
async fn commit_status_rpc_acl_gates() {
    assert!(
        false,
        "TODO 13-04: Write+ create / Read+ list; soft not_found for private ACL"
    );
}
