//! GIT-04 Wave 0 stubs: sshKey.add / list / revoke + verified gate.
//!
//! RED until sshKey.* RPC handlers land (09-02+). Do not implement production handlers here.
//! Threat mitigations encoded in assertions: T-09-02 (require_verified on add; fingerprint unique — D-SSH-05).

/// Verified `sshKey.add` with title + ed25519 OpenSSH line returns list item with SHA256 fingerprint
/// and no secret/private-key reveal (D-SSH-05).
#[tokio::test]
async fn ssh_key_add_verified_ed25519_returns_fingerprint() {
    assert!(
        false,
        "Wave 0: verified sshKey.add (title + ed25519) must return ok + fingerprint SHA256:…; no secret reveal (GIT-04 / D-SSH-05)"
    );
}

/// `sshKey.list` after add includes the new key (created_at DESC — ASSUME GIT-04 ordering).
#[tokio::test]
async fn ssh_key_list_after_add() {
    assert!(
        false,
        "Wave 0: sshKey.list after add must include the key ordered by created_at DESC (GIT-04)"
    );
}

/// `sshKey.revoke` hard-deletes; key absent from subsequent list (ASSUME hard-delete).
#[tokio::test]
async fn ssh_key_revoke_removes_from_list() {
    assert!(
        false,
        "Wave 0: after sshKey.revoke, key id must be absent from sshKey.list; unknown/other-user → sshKey.not_found (GIT-04)"
    );
}

/// Unverified session cannot add keys → `auth.email_unverified` (require_verified / D-SSH-05).
#[tokio::test]
async fn ssh_key_add_unverified_email_unverified() {
    assert!(
        false,
        "Wave 0: unverified sshKey.add → auth.email_unverified (GIT-04 / D-SSH-05 / T-09-02)"
    );
}

/// Empty / whitespace title on add → `sshKey.title_required` (D-SSH-05).
#[tokio::test]
async fn ssh_key_add_empty_title_required() {
    assert!(
        false,
        "Wave 0: empty title on sshKey.add → sshKey.title_required (GIT-04 / D-SSH-05)"
    );
}

/// Duplicate fingerprint (same public key) → domain uniqueness error (D-SSH-05 / T-09-02).
#[tokio::test]
async fn ssh_key_add_duplicate_fingerprint_rejected() {
    assert!(
        false,
        "Wave 0: duplicate fingerprint on sshKey.add → domain uniqueness error (GIT-04 / D-SSH-05 / T-09-02)"
    );
}

/// 26th key for one user → max 25 rejected (GIT-04 / D-SSH-05).
#[tokio::test]
async fn ssh_key_add_26th_key_max_25() {
    assert!(
        false,
        "Wave 0: 26th sshKey.add for one user must fail (max 25 keys) (GIT-04 / D-SSH-05)"
    );
}
