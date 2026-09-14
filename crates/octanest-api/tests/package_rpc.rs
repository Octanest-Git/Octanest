//! Phase 20 Wave 0 stubs — packages.list / packages.deleteVersion RPC (PKG-05).

#![allow(dead_code)]

/// packages.list by owner returns packages for that user/org.
#[tokio::test]
async fn package_rpc_list_by_owner() {
    assert!(false, "expected packages.list by owner");
}

/// packages.list by repository_id returns repo-linked packages.
#[tokio::test]
async fn package_rpc_list_by_repo_link() {
    assert!(false, "expected packages.list filtered by repository_id");
}

/// packages.deleteVersion requires Admin session + confirm name@version.
#[tokio::test]
async fn package_rpc_delete_version_admin_confirm() {
    assert!(
        false,
        "expected packages.deleteVersion Admin session + confirm name@version (D-PKG-12)"
    );
}

/// Wrong confirm string is rejected.
#[tokio::test]
async fn package_rpc_delete_version_bad_confirm_rejected() {
    assert!(false, "expected reject when confirm != name@version");
}
