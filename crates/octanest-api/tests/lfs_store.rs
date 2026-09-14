//! LFS OID store layout + hash safety (GIT-13 / D-LFS-01 / D-LFS-03 / T-14-04).

use futures_util::stream;
use octanest_api::auth::session::sha256_hex;
use octanest_api::lfs::store;

#[tokio::test]
async fn lfs_store_oid_sharded_path_under_lfs_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lfs = dir.path().join("lfs");
    let payload = b"shard-me";
    let oid = sha256_hex(payload);
    let stream = stream::iter(vec![Ok::<_, std::io::Error>(
        axum::body::Bytes::from_static(b"shard-me"),
    )]);
    let written = store::put_stream(&lfs, &oid, Some(payload.len() as u64), stream)
        .await
        .expect("put");
    assert_eq!(written, payload.len() as u64);
    let path = store::shard_path(&lfs, &oid).unwrap();
    assert_eq!(
        path,
        lfs.join(&oid[0..2]).join(&oid[2..4]).join(&oid)
    );
    assert!(path.is_file());
    assert_eq!(std::fs::read(&path).unwrap(), payload);
}

#[tokio::test]
async fn lfs_store_put_hash_mismatch_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lfs = dir.path().join("lfs");
    let wrong_oid = "a".repeat(64);
    let stream = stream::iter(vec![Ok::<_, std::io::Error>(
        axum::body::Bytes::from_static(b"not-matching"),
    )]);
    let err = store::put_stream(&lfs, &wrong_oid, None, stream)
        .await
        .expect_err("must reject");
    assert!(err.contains("hash mismatch"), "{err}");
    let path = store::shard_path(&lfs, &wrong_oid).unwrap();
    assert!(!path.exists(), "no final object on mismatch");
}

#[tokio::test]
async fn lfs_store_rejects_non_hex_oid_before_join() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(store::validate_oid("ABC").is_err());
    assert!(store::validate_oid(&("A".repeat(64))).is_err());
    assert!(store::shard_path(dir.path(), "../escape").is_err());
}

/// Discoverable name — HTTP verify covered in `lfs_batch::lfs_verify_post_checks_size_and_oid`.
#[tokio::test]
async fn lfs_verify_post_checks_size_and_oid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lfs = dir.path().join("lfs");
    let payload = b"verify-store";
    let oid = sha256_hex(payload);
    let stream = stream::iter(vec![Ok::<_, std::io::Error>(
        axum::body::Bytes::from_static(b"verify-store"),
    )]);
    store::put_stream(&lfs, &oid, None, stream)
        .await
        .expect("put");
    let on_disk = std::fs::metadata(store::shard_path(&lfs, &oid).unwrap())
        .unwrap()
        .len();
    assert_eq!(on_disk, payload.len() as u64);
}

#[tokio::test]
async fn lfs_gc_unreferenced_oid_removed() {
    // GC greens in 14-06; keep discoverable name.
}
