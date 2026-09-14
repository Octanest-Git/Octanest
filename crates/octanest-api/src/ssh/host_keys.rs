//! SSH host key load/generate (Ed25519) under `OCTANEST_SSH_HOST_KEY_DIR`.

use std::path::{Path, PathBuf};

use russh::keys::{Algorithm, PrivateKey};

const HOST_KEY_FILE: &str = "ssh_host_ed25519_key";

/// Resolve host-key directory (`OCTANEST_SSH_HOST_KEY_DIR`, default `var/ssh`).
pub fn host_key_dir() -> PathBuf {
    std::env::var("OCTANEST_SSH_HOST_KEY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("var/ssh"))
}

/// Load existing Ed25519 host key or generate + persist one.
pub async fn load_or_generate(dir: &Path) -> Result<PrivateKey, String> {
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| format!("create host key dir: {e}"))?;
    let path = dir.join(HOST_KEY_FILE);
    if tokio::fs::try_exists(&path).await.unwrap_or(false) {
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| format!("read host key: {e}"))?;
        let pem = String::from_utf8(bytes).map_err(|e| format!("host key utf8: {e}"))?;
        return PrivateKey::from_openssh(&pem).map_err(|e| format!("parse host key: {e}"));
    }

    let key = PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519)
        .map_err(|e| format!("generate host key: {e}"))?;
    let pem = key
        .to_openssh(russh::keys::ssh_key::LineEnding::LF)
        .map_err(|e| format!("encode host key: {e}"))?;
    tokio::fs::write(&path, pem.as_bytes())
        .await
        .map_err(|e| format!("write host key: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await;
    }
    Ok(key)
}
