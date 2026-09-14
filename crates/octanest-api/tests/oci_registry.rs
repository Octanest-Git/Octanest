//! Phase 20 Wave 0 stubs — OCI Distribution Spec (PKG-01 / D-PKG-15).
//! Greened by later plans (protocol handlers + ACL). Cookie ignored on registry paths (T-20-01).

#![allow(dead_code)]

/// GET /v2/ discovery probe (Docker/podman registry handshake).
#[tokio::test]
async fn oci_registry_v2_discovery() {
    assert!(
        false,
        "expected GET /v2/ → 200 or 401 with Docker-Distribution-API-Version; no SPA HTML"
    );
}

/// Anonymous pull of public blob/manifest succeeds without credentials.
#[tokio::test]
async fn oci_registry_anonymous_public_pull() {
    assert!(
        false,
        "expected anonymous GET blob/manifest for public package to succeed (D-PKG-05)"
    );
}

/// Authenticated push of blob + manifest with PAT Basic/Bearer (not session Cookie).
#[tokio::test]
async fn oci_registry_auth_push_manifest_blob() {
    assert!(
        false,
        "expected PAT Basic/Bearer push of blob+manifest; Cookie header ignored"
    );
}

/// Tags list endpoint returns repository tags.
#[tokio::test]
async fn oci_registry_tags_list() {
    assert!(false, "expected GET /v2/<name>/tags/list to return tags");
}

/// Digest-addressed manifests are immutable (conflict on overwrite).
#[tokio::test]
async fn oci_registry_digest_immutable_conflict() {
    assert!(
        false,
        "expected digest put conflict when bytes differ (D-PKG-10)"
    );
}

/// Tags may be retargeted to a new digest.
#[tokio::test]
async fn oci_registry_tag_retarget_allowed() {
    assert!(false, "expected tag PUT to retarget digest (D-PKG-10)");
}

/// Manifest delete requires Admin capability + package:write.
#[tokio::test]
async fn oci_registry_manifest_delete_admin() {
    assert!(
        false,
        "expected DELETE manifest only for Admin + package:write (D-PKG-06)"
    );
}

/// Session Cookie on /v2 must not authenticate registry clients.
#[tokio::test]
async fn oci_registry_cookie_header_ignored() {
    assert!(
        false,
        "expected Cookie alone to be ignored on /v2 (Phase 8 D-12 lesson)"
    );
}
