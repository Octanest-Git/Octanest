//! Parse SSH `exec` pack commands and spawn system `git-upload-pack` (no shell).

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::process::Command;

use crate::git::bare_repo_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackCommand {
    UploadPack { owner: String, name: String },
    /// Deferred to 09-04 for full ACL/verify.
    ReceivePack { owner: String, name: String },
}

/// Parse `git-upload-pack 'owner/name.git'` / `git-receive-pack "owner/name.git"`.
pub fn parse_pack_exec(data: &[u8]) -> Option<PackCommand> {
    let s = std::str::from_utf8(data).ok()?.trim();
    let (kind, rest) = if let Some(r) = s.strip_prefix("git-upload-pack") {
        ("upload", r.trim())
    } else if let Some(r) = s.strip_prefix("git-receive-pack") {
        ("receive", r.trim())
    } else {
        return None;
    };

    let path = strip_quotes(rest)?;
    let path = path.trim_start_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, name) = path.split_once('/')?;
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return None;
    }
    match kind {
        "upload" => Some(PackCommand::UploadPack {
            owner: owner.to_string(),
            name: name.to_string(),
        }),
        "receive" => Some(PackCommand::ReceivePack {
            owner: owner.to_string(),
            name: name.to_string(),
        }),
        _ => None,
    }
}

fn strip_quotes(s: &str) -> Option<&str> {
    let s = s.trim();
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        Some(&s[1..s.len() - 1])
    } else if !s.is_empty() {
        Some(s)
    } else {
        None
    }
}

pub fn resolve_bare(repos_dir: &Path, owner: &str, name: &str) -> Result<PathBuf, String> {
    bare_repo_path(repos_dir, owner, name).map_err(|e| e.message)
}

/// Spawn `git-upload-pack <bare>` with argv only (no shell) and bridge stdio.
pub async fn run_upload_pack<R, W, E>(
    bare: &Path,
    mut stdin_rx: R,
    mut stdout_tx: W,
    mut stderr_tx: E,
) -> Result<i32, String>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
    E: AsyncWrite + Unpin,
{
    let mut child = Command::new("git")
        .arg("upload-pack")
        .arg(bare)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("spawn git-upload-pack: {e}"))?;

    let mut child_stdin = child.stdin.take().ok_or("missing stdin")?;
    let mut child_stdout = child.stdout.take().ok_or("missing stdout")?;
    let mut child_stderr = child.stderr.take().ok_or("missing stderr")?;

    let stdin_pipe = async {
        let _ = tokio::io::copy(&mut stdin_rx, &mut child_stdin).await;
        let _ = child_stdin.shutdown().await;
    };
    let stdout_pipe = async {
        let _ = tokio::io::copy(&mut child_stdout, &mut stdout_tx).await;
        let _ = stdout_tx.flush().await;
    };
    let stderr_pipe = async {
        let _ = tokio::io::copy(&mut child_stderr, &mut stderr_tx).await;
        let _ = stderr_tx.flush().await;
    };

    tokio::join!(stdin_pipe, stdout_pipe, stderr_pipe);

    let status = child
        .wait()
        .await
        .map_err(|e| format!("wait git-upload-pack: {e}"))?;
    Ok(status.code().unwrap_or(1))
}
