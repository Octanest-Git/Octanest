//! Wave 0 stubs for Git LFS Batch API / auth / enable / quota / dedup (GIT-12, D-LFS-*).
//! Bodies are placeholders; later plans (14-02+) replace with real assertions.

/// GIT-12: POST …/info/lfs/objects/batch upload → PUT → GET download happy path.
#[tokio::test]
async fn lfs_batch_upload_put_download_happy_path() {
    // Wave 0: discoverable stub — implement batch+basic transfer in 14-02+.
}

/// D-LFS-09: Cookie alone must not authenticate LFS (PAT Basic only).
#[tokio::test]
async fn lfs_batch_cookie_ignored_as_anon() {
    // Wave 0: Cookie session must not authorize LFS batch (mirror Smart HTTP).
}

/// D-LFS-09: private unauth → 401 with LFS-Authenticate: Basic realm="Git LFS".
#[tokio::test]
async fn lfs_batch_unauth_private_returns_401_lfs_authenticate() {
    // Wave 0: unauthenticated private LFS batch → 401 + LFS-Authenticate Basic.
}

/// D-LFS-11: insufficient PAT scope → 403.
#[tokio::test]
async fn lfs_batch_insufficient_pat_scope_forbidden() {
    // Wave 0: PAT without contents/repo scope → 403 on LFS batch.
}

/// D-LFS-09: Write + verified required for upload batch actions.
#[tokio::test]
async fn lfs_batch_upload_requires_write_and_verified() {
    // Wave 0: Read-only PAT must not receive upload actions; unverified blocked.
}

/// D-LFS-10: disabled repo rejects batch (lfs_enable filter).
#[tokio::test]
async fn lfs_enable_disabled_repo_rejects_batch() {
    // Wave 0: repositories.lfs_enabled=false → clear LFS error on batch (14-04).
}

/// D-LFS-14: over-quota reject (lfs_quota filter).
#[tokio::test]
async fn lfs_quota_over_quota_upload_rejected() {
    // Wave 0: upload exceeding max size / repo / user quota → reject (14-07).
}

/// D-LFS-02: existing OID omits upload actions (lfs_dedup filter).
#[tokio::test]
async fn lfs_dedup_existing_oid_omits_upload_actions() {
    // Wave 0: batch for already-stored OID must omit upload href (dedup).
}
