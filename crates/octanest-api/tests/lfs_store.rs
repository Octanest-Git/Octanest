//! Wave 0 stubs for LFS OID store layout, verify, and GC (GIT-13, D-LFS-01/03/15).
//! Bodies are placeholders; greened by plans 14-05 / 14-06.

/// D-LFS-01 / D-LFS-03: OID-sharded path under OCTANEST_LFS_DIR — `{ab}/{cd}/{oid}`.
#[tokio::test]
async fn lfs_store_oid_sharded_path_under_lfs_dir() {
    // Wave 0: PUT must write OCTANEST_LFS_DIR/ab/cd/<64-hex-oid> (14-05).
}

/// Hash mismatch on PUT must reject (object not committed).
#[tokio::test]
async fn lfs_store_put_hash_mismatch_rejected() {
    // Wave 0: PUT body SHA-256 ≠ oid → reject; no durable object row.
}

/// Optional verify endpoint / size check (D-LFS-07 resumable-within-basic).
#[tokio::test]
async fn lfs_verify_post_checks_size_and_oid() {
    // Wave 0: POST verify action confirms size+oid before marking complete.
}

/// D-LFS-15: refcount GC removes unreferenced OIDs (lfs_gc filter).
#[tokio::test]
async fn lfs_gc_unreferenced_oid_removed() {
    // Wave 0: OID with refcount 0 deleted from disk + lfs_objects on GC (14-06).
}
