//! Phase 20 Wave 0 stubs — generic/raw registry (PKG-03 / D-PKG-16).

#![allow(dead_code)]

/// PUT /generic/<owner>/<name>/<version>/<file> uploads a file.
#[tokio::test]
async fn generic_registry_put_file() {
    assert!(
        false,
        "expected PUT /generic/<owner>/<name>/<version>/<file> with PAT auth"
    );
}

/// GET downloads a previously uploaded file.
#[tokio::test]
async fn generic_registry_get_file() {
    assert!(false, "expected GET to return stored bytes");
}

/// DELETE version removes all files for that version id.
#[tokio::test]
async fn generic_registry_delete_version() {
    assert!(
        false,
        "expected DELETE version to remove files (Admin + package:write)"
    );
}

/// Overwrite of an existing version returns 409 (immutable versions).
#[tokio::test]
async fn generic_registry_overwrite_conflict() {
    assert!(
        false,
        "expected 409 on version/file overwrite while present (D-PKG-10)"
    );
}

/// List versions/files for a generic package.
#[tokio::test]
async fn generic_registry_list() {
    assert!(false, "expected list of versions/files for owner/name");
}
