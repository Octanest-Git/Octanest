//! Phase 20 — hybrid package ACL ∩ PAT (PKG-04 / D-PKG-04..06).
//! Helper-level matrix (protocol handlers greened in later plans).

use octanest_api::packages::acl::{
    authorize, classic_repo_alone_denied, pat_allows_packages, PackageAction,
};
use octanest_api::repo::Capability;
use octanest_core::ClassicPatScope;

#[tokio::test]
async fn package_acl_anonymous_public_pull_ok() {
    assert!(authorize("public", None, false, PackageAction::Pull));
}

#[tokio::test]
async fn package_acl_private_pull_needs_read_and_package_read() {
    assert!(!authorize(
        "private",
        Some(Capability::Read),
        false,
        PackageAction::Pull
    ));
    assert!(authorize(
        "private",
        Some(Capability::Read),
        true,
        PackageAction::Pull
    ));
    assert!(pat_allows_packages(
        Some(&[ClassicPatScope::PackageRead]),
        None,
        PackageAction::Pull
    ));
}

#[tokio::test]
async fn package_acl_publish_needs_write_and_package_write() {
    assert!(!authorize(
        "private",
        Some(Capability::Write),
        false,
        PackageAction::Publish
    ));
    assert!(authorize(
        "private",
        Some(Capability::Write),
        true,
        PackageAction::Publish
    ));
    assert!(pat_allows_packages(
        Some(&[ClassicPatScope::PackageWrite]),
        None,
        PackageAction::Publish
    ));
}

#[tokio::test]
async fn package_acl_delete_needs_admin_and_package_write() {
    assert!(!authorize(
        "private",
        Some(Capability::Write),
        true,
        PackageAction::Delete
    ));
    assert!(authorize(
        "private",
        Some(Capability::Admin),
        true,
        PackageAction::Delete
    ));
}

#[tokio::test]
async fn package_acl_classic_repo_scope_alone_denied() {
    assert!(!pat_allows_packages(
        Some(&[ClassicPatScope::Repo]),
        None,
        PackageAction::Pull
    ));
    assert!(classic_repo_alone_denied(&[ClassicPatScope::Repo]));
}
