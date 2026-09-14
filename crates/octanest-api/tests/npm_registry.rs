//! Phase 20 Wave 0 stubs — npm registry API (PKG-02 / D-PKG-14).

#![allow(dead_code)]

/// Packument GET for /npm/<owner>/<name>.
#[tokio::test]
async fn npm_registry_packument_get() {
    assert!(false, "expected packument GET under /npm/<owner>/");
}

/// Publish PUT with attachment (tarball) succeeds with package:write.
#[tokio::test]
async fn npm_registry_publish_put_attachment() {
    assert!(
        false,
        "expected PUT publish with _attachments; auth via PAT Bearer/Basic"
    );
}

/// Tarball GET uses absolute OCTANEST_PUBLIC_ORIGIN URL from packument.
#[tokio::test]
async fn npm_registry_tarball_get_public_origin() {
    assert!(
        false,
        "expected dist.tarball under OCTANEST_PUBLIC_ORIGIN /npm/.../-/....tgz"
    );
}

/// Dist-tags get/set under /- /package/dist-tags.
#[tokio::test]
async fn npm_registry_dist_tags() {
    assert!(false, "expected dist-tags read/write (D-PKG-14)");
}

/// Deprecate version metadata.
#[tokio::test]
async fn npm_registry_deprecate() {
    assert!(false, "expected deprecate flow (D-PKG-14)");
}

/// Search via /-/v1/search.
#[tokio::test]
async fn npm_registry_search_v1() {
    assert!(false, "expected GET /npm/.../-/v1/search");
}

/// Version overwrite while present returns conflict (D-PKG-10).
#[tokio::test]
async fn npm_registry_version_overwrite_conflict() {
    assert!(
        false,
        "expected 409 / EPUBLISHCONFLICT on version overwrite (D-PKG-10)"
    );
}
