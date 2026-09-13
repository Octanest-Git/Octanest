//! GIT-03 Wave 0 stubs: Git-over-SSH auth / ACL / pack allowlist / rate-limit.
//!
//! RED until russh listener + pack bridge land (09-04 / 09-05). Do not implement the listener here.
//! Threat mitigations: T-09-01 — force user git; reject shell/pty; private non-owner denied (D-SSH-03, D-SSH-04).

/// SSH username other than `git` is rejected (D-SSH-03).
#[tokio::test]
async fn git_ssh_username_other_than_git_rejected() {
    assert!(
        false,
        "Wave 0: SSH username ≠ git must be rejected; identity is fingerprint only (GIT-03 / D-SSH-03 / T-09-01)"
    );
}

/// Registered public key + username `git` is accepted (D-SSH-03).
#[tokio::test]
async fn git_ssh_registered_key_user_git_accepted() {
    assert!(
        false,
        "Wave 0: registered key + user git must authenticate (GIT-03 / D-SSH-03)"
    );
}

/// Public repo: authenticated `git-upload-pack` happy-path stub (D-SSH-04; A1 — key required).
#[tokio::test]
async fn git_ssh_public_upload_pack_happy_path() {
    assert!(
        false,
        "Wave 0: registered key must allow git-upload-pack on public repo (GIT-03 / D-SSH-04)"
    );
}

/// Private non-owner denied with clear git stderr (not HTTP 401/404) (D-SSH-04 / T-09-01).
#[tokio::test]
async fn git_ssh_private_non_owner_git_stderr_deny() {
    assert!(
        false,
        "Wave 0: private non-owner → clear git stderr deny (GIT-03 / D-SSH-04 / T-09-01)"
    );
}

/// Unverified email cannot push (`git-receive-pack`) (D-SSH-04).
#[tokio::test]
async fn git_ssh_push_unverified_email_denied() {
    assert!(
        false,
        "Wave 0: unverified email cannot git-receive-pack over SSH (GIT-03 / D-SSH-04)"
    );
}

/// Non-pack exec (shell / pty / sftp) rejected — pack allowlist only (D-SSH-01 / T-09-01).
#[tokio::test]
async fn git_ssh_non_pack_exec_shell_rejected() {
    assert!(
        false,
        "Wave 0: shell/pty/non pack exec must be rejected; only git-upload-pack / git-receive-pack (GIT-03 / D-SSH-01 / T-09-01)"
    );
}

/// Failed pubkey auth over limit is rate-limited (D-SSH-07).
#[tokio::test]
async fn git_ssh_failed_pubkey_rate_limited() {
    assert!(
        false,
        "Wave 0: failed pubkey over limit must be rate-limited (IP / fingerprint) (GIT-03 / D-SSH-07)"
    );
}
