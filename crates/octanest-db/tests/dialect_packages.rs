//! Phase 20 Wave 0 stub — packages migration parity (greened by 20-02).
//!
//! Expects tri-dialect migration `00xx_packages` (id resolved at execute) with:
//! packages, package_versions, package_blobs, package_blob_refs, package_quota_overrides.

use std::path::PathBuf;

fn migration_candidates() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations/sqlite");
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with("_packages.sql") {
                out.push(e.path());
            }
        }
    }
    out
}

/// Wave 0: assert expected object names once migration exists; until then ignore.
#[tokio::test]
async fn dialect_packages_schema_presence() {
    let paths = migration_candidates();
    assert!(
        !paths.is_empty(),
        "expected */migrations/*/00xx_packages.sql (id resolved at execute)"
    );
    let sql = std::fs::read_to_string(&paths[0]).unwrap_or_default();
    for table in [
        "packages",
        "package_versions",
        "package_blobs",
        "package_blob_refs",
        "package_quota_overrides",
    ] {
        assert!(
            sql.contains(table),
            "packages migration must define {}",
            table
        );
    }
}
