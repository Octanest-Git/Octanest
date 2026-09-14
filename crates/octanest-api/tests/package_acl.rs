//! Phase 20 Wave 0 stubs — hybrid package ACL ∩ PAT (PKG-04 / D-PKG-04..06).

#![allow(dead_code)]

/// Anonymous pull of public packages succeeds.
#[tokio::test]
async fn package_acl_anonymous_public_pull_ok() {
    assert!(false, "expected anonymous public pull OK (D-PKG-05)");
}

/// Private pull requires Read+ ACL and package:read PAT scope.
#[tokio::test]
async fn package_acl_private_pull_needs_read_and_package_read() {
    assert!(
        false,
        "expected private pull to require Capability::Read+ and package:read"
    );
}

/// Publish requires Write+ ACL and package:write PAT scope.
#[tokio::test]
async fn package_acl_publish_needs_write_and_package_write() {
    assert!(
        false,
        "expected publish to require Capability::Write+ and package:write (D-PKG-06)"
    );
}

/// Delete requires Admin ACL and package:write.
#[tokio::test]
async fn package_acl_delete_needs_admin_and_package_write() {
    assert!(
        false,
        "expected delete to require Capability::Admin and package:write"
    );
}

/// Classic `repo` scope alone does not grant package access (fail closed).
#[tokio::test]
async fn package_acl_classic_repo_scope_alone_denied() {
    assert!(
        false,
        "expected classic repo scope alone denied for packages (T-20-02 / D-PKG-04)"
    );
}
