//! Parse SSH `exec` pack commands, ACL, and spawn system git pack helpers (no shell).

use std::path::{Path, PathBuf};
use std::process::Stdio;

use octanest_db::Database;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::process::Command;

use crate::git::bare_repo_path;
use crate::repo::{
    effective_capability, is_private_visibility, lookup_repo_row_or_redirect, meets, Capability,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackCommand {
    UploadPack { owner: String, name: String },
    ReceivePack { owner: String, name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackAction {
    Fetch,
    Push,
}

#[derive(Debug)]
pub enum AuthzDecision {
    Allow { bare: PathBuf },
    Deny { message: String },
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

/// Authorize pack access using the same owner/visibility helpers as Smart HTTP (D-SSH-04).
pub async fn authorize_pack(
    db: &Database,
    repos_dir: &Path,
    caller_user_id: &str,
    cmd: &PackCommand,
) -> AuthzDecision {
    let (owner, name, action) = match cmd {
        PackCommand::UploadPack { owner, name } => (owner.as_str(), name.as_str(), PackAction::Fetch),
        PackCommand::ReceivePack { owner, name } => (owner.as_str(), name.as_str(), PackAction::Push),
    };

    let pair = match lookup_repo_row_or_redirect(db, owner, name).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return AuthzDecision::Deny {
                message: "ERROR: Repository not found.\n".into(),
            };
        }
        Err(_) => {
            return AuthzDecision::Deny {
                message: "ERROR: Internal error.\n".into(),
            };
        }
    };
    let (row, owner_ref) = pair;
    let disk_owner = owner_ref.slug();
    let disk_name = row.name.as_str();

    let bare = match resolve_bare(repos_dir, disk_owner, disk_name) {
        Ok(p) if p.exists() => p,
        _ => {
            return AuthzDecision::Deny {
                message: "ERROR: Repository not found.\n".into(),
            };
        }
    };

    // Match Smart HTTP: Capability coalesce (personal owner / org role / collaborator / public).
    let capability = match effective_capability(db, Some(caller_user_id), &row, &owner_ref).await {
        Ok(c) => c,
        Err(_) => {
            return AuthzDecision::Deny {
                message: "ERROR: Internal error.\n".into(),
            };
        }
    };

    match action {
        PackAction::Fetch => {
            if is_private_visibility(&row.visibility) && !meets(capability, Capability::Read) {
                return AuthzDecision::Deny {
                    message: "ERROR: Permission denied to this repository.\n".into(),
                };
            }
            AuthzDecision::Allow { bare }
        }
        PackAction::Push => {
            if !meets(capability, Capability::Write) {
                return AuthzDecision::Deny {
                    message: "ERROR: Permission denied to this repository.\n".into(),
                };
            }
            let caller = match db.find_user_by_id(caller_user_id).await {
                Ok(Some(u)) => u,
                _ => {
                    return AuthzDecision::Deny {
                        message: "ERROR: Permission denied to this repository.\n".into(),
                    };
                }
            };
            if caller
                .email_verified_at
                .as_deref()
                .filter(|s| !s.is_empty())
                .is_none()
            {
                return AuthzDecision::Deny {
                    message: "ERROR: Email verification required to push.\n".into(),
                };
            }
            AuthzDecision::Allow { bare }
        }
    }
}

/// Spawn `git-upload-pack` or `git-receive-pack` with argv only (no shell) and bridge stdio.
pub async fn run_pack_command<R, W, E>(
    program: &str,
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
        .arg(program)
        .arg(bare)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("spawn git {program}: {e}"))?;

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
        .map_err(|e| format!("wait git {program}: {e}"))?;
    Ok(status.code().unwrap_or(1))
}

/// Write a denial message to the channel as git stderr (extended data 1) then close.
pub async fn write_git_stderr_deny(
    handle: &russh::server::Handle,
    channel: russh::ChannelId,
    message: &str,
) {
    let _ = handle
        .extended_data(channel, 1, message.as_bytes().to_vec())
        .await;
    let _ = handle.eof(channel).await;
    let _ = handle.close(channel).await;
}
