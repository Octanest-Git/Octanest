//! SSH pubkey → fingerprint → registered user (D-SSH-03).

use octanest_db::{Database, SshKeyRow};
use ssh_key::{HashAlg, PublicKey};

/// OpenSSH-display fingerprint (`SHA256:…`) for a russh/ssh-key public key.
pub fn fingerprint_of(key: &PublicKey) -> String {
    key.fingerprint(HashAlg::Sha256).to_string()
}

/// Look up a registered SSH key by fingerprint (authentication usage only).
pub async fn find_registered_key(
    db: &Database,
    key: &PublicKey,
) -> Result<Option<SshKeyRow>, String> {
    let fp = fingerprint_of(key);
    match db.find_ssh_key_by_fingerprint(&fp).await? {
        Some(row) if row.can_authenticate => Ok(Some(row)),
        _ => Ok(None),
    }
}
