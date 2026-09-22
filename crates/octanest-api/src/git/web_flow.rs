//! Instance “web-flow” Ed25519 SSH signing key for forge-authored seed commits.
//!
//! Key pair lives next to the SSH host key under `OCTANEST_SSH_HOST_KEY_DIR`
//! (default `var/ssh`): private `web-flow`, public `web-flow.pub`.

use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

use crate::ssh::host_keys;

const WEB_FLOW_KEY_BASENAME: &str = "web-flow";

/// Directory for the web-flow key (same as SSH host keys).
pub fn web_flow_dir() -> PathBuf {
    host_keys::host_key_dir()
}

/// Private key path: `{ssh_host_key_dir}/web-flow`.
pub fn private_key_path() -> PathBuf {
    web_flow_dir().join(WEB_FLOW_KEY_BASENAME)
}

/// Public key path: `{ssh_host_key_dir}/web-flow.pub`.
pub fn public_key_path() -> PathBuf {
    web_flow_dir().join(format!("{WEB_FLOW_KEY_BASENAME}.pub"))
}

fn is_dev_env() -> bool {
    let env = std::env::var("OCTANEST_ENV").unwrap_or_else(|_| "development".into());
    matches!(
        env.to_ascii_lowercase().as_str(),
        "development" | "dev" | "compose" | "test"
    )
}

/// Resolve against process cwd when relative.
///
/// Seed commits run `git -C <tmpdir> … -c user.signingkey=…`; a relative key
/// path would resolve under the temp worktree, not the package cwd.
fn absolute_key_path(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path);
    }
    let cwd = std::env::current_dir().map_err(|e| format!("cwd for web-flow key: {e}"))?;
    Ok(cwd.join(path))
}

/// Ensure the Ed25519 web-flow key exists.
///
/// Returns an **absolute** private-key path so `git -C <tmpdir> commit -S` can
/// load it (relative paths resolve under the worktree).
///
/// - Development / compose / test: generate with `ssh-keygen` if missing.
/// - Production / cloud: fail closed if missing (ops must provision the key).
pub async fn ensure_web_flow_key() -> Result<PathBuf, String> {
    let dir = absolute_key_path(web_flow_dir())?;
    let priv_path = dir.join(WEB_FLOW_KEY_BASENAME);
    let pub_path = dir.join(format!("{WEB_FLOW_KEY_BASENAME}.pub"));

    if tokio::fs::try_exists(&priv_path).await.unwrap_or(false)
        && tokio::fs::try_exists(&pub_path).await.unwrap_or(false)
    {
        return Ok(priv_path);
    }

    if !is_dev_env() {
        return Err(format!(
            "web-flow signing key missing at {} (and {}.pub); provision under OCTANEST_SSH_HOST_KEY_DIR",
            priv_path.display(),
            WEB_FLOW_KEY_BASENAME
        ));
    }

    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("create web-flow key dir: {e}"))?;

    // Remove partial files before generating.
    let _ = tokio::fs::remove_file(&priv_path).await;
    let _ = tokio::fs::remove_file(&pub_path).await;

    let out = Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-N",
            "",
            "-f",
            priv_path
                .to_str()
                .ok_or_else(|| "non-utf8 web-flow key path".to_string())?,
            "-C",
            "octanest-web-flow",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to spawn ssh-keygen: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("ssh-keygen failed: {}", stderr.trim()));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = tokio::fs::set_permissions(&priv_path, std::fs::Permissions::from_mode(0o600)).await;
    }

    Ok(priv_path)
}

/// Read the OpenSSH public key line (`ssh-ed25519 AAAA… comment`), if present.
pub async fn public_key_line() -> Result<Option<String>, String> {
    let path = public_key_path();
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok(None);
    }
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("read web-flow.pub: {e}"))?;
    let line = String::from_utf8(bytes)
        .map_err(|e| format!("web-flow.pub utf8: {e}"))?
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if line.is_empty() {
        Ok(None)
    } else {
        Ok(Some(line))
    }
}

/// Public key bytes (OpenSSH `.pub` file contents), if present.
pub async fn public_key_bytes() -> Result<Option<Vec<u8>>, String> {
    let path = public_key_path();
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok(None);
    }
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("read web-flow.pub: {e}"))?;
    Ok(Some(bytes))
}

/// True when both private and public key files exist.
pub async fn key_present() -> bool {
    let priv_ok = tokio::fs::try_exists(private_key_path())
        .await
        .unwrap_or(false);
    let pub_ok = tokio::fs::try_exists(public_key_path())
        .await
        .unwrap_or(false);
    priv_ok && pub_ok
}
