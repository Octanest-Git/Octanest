//! Build git `allowedSignersFile` + temp GNUPGHOME for commit signature verification.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use octanest_db::Database;
use tokio::io::AsyncWriteExt;

use super::author_resolve;

/// Materials kept alive for the duration of a verify request.
pub struct VerifyKeyring {
    _signers_dir: Option<tempfile::TempDir>,
    pub allowed_signers: Option<PathBuf>,
    _gpg_dir: Option<tempfile::TempDir>,
    pub gpg_home: Option<PathBuf>,
}

impl VerifyKeyring {
    pub fn empty() -> Self {
        Self {
            _signers_dir: None,
            allowed_signers: None,
            _gpg_dir: None,
            gpg_home: None,
        }
    }

    pub fn has_any(&self) -> bool {
        self.allowed_signers.is_some() || self.gpg_home.is_some()
    }
}

/// Collect allowed_signers + GNUPGHOME for the given committer/author emails.
pub async fn keyring_for_emails(db: &Database, emails: &[String]) -> VerifyKeyring {
    let resolved = author_resolve::resolve_authors_for_emails(db, emails).await;
    let mut out = VerifyKeyring::empty();

    if let Some(named) = author_resolve::build_allowed_signers_file(db, emails, &resolved).await {
        if let Ok(dir) = tempfile::TempDir::new() {
            let path = dir.path().join("allowed_signers");
            if tokio::fs::copy(named.path(), &path).await.is_ok() {
                out.allowed_signers = Some(path);
                out._signers_dir = Some(dir);
            }
        }
    }

    if let Some(home) = build_gpg_home(db, &resolved).await {
        out.gpg_home = Some(home.path().to_path_buf());
        out._gpg_dir = Some(home);
    }

    out
}

/// Backward-compatible helper used by older call sites.
#[allow(dead_code)]
pub async fn allowed_signers_for_emails(
    db: &Database,
    emails: &[String],
) -> Option<(tempfile::TempDir, PathBuf)> {
    let ring = keyring_for_emails(db, emails).await;
    match (ring._signers_dir, ring.allowed_signers) {
        (Some(dir), Some(path)) => {
            // path is inside dir; return dir + path
            Some((dir, path))
        }
        _ => None,
    }
}

async fn build_gpg_home(
    db: &Database,
    resolved: &std::collections::HashMap<String, author_resolve::ResolvedAuthor>,
) -> Option<tempfile::TempDir> {
    let mut seen = std::collections::HashSet::new();
    let mut armors: Vec<String> = Vec::new();
    for author in resolved.values() {
        let Some(uid) = author.user_id.as_deref() else {
            continue;
        };
        if !seen.insert(uid.to_string()) {
            continue;
        }
        let Ok(keys) = db.list_gpg_keys_for_user(uid).await else {
            continue;
        };
        for k in keys {
            armors.push(k.armored_public_key);
        }
    }
    if armors.is_empty() {
        return None;
    }

    let dir = tempfile::TempDir::new().ok()?;
    // GnuPG refuses world-writable homes.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700));
    }

    for armor in &armors {
        let mut child = tokio::process::Command::new("gpg")
            .args(["--batch", "--yes", "--import"])
            .env("GNUPGHOME", dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(armor.as_bytes()).await;
        }
        let _ = child.wait().await;
    }
    Some(dir)
}

/// Apply forge Verified policy after crypto `%G?` succeeded.
///
/// - Committer email must resolve to a user with a verified account email (or noreply).
/// - For GPG signatures, committer email must appear in a registered key UID.
pub async fn apply_verified_policy(
    db: &Database,
    committer_email: &str,
    author_email: &str,
    signature_status: &str,
    signature_kind: &str,
) -> String {
    if signature_status != "valid" {
        return signature_status.to_string();
    }
    let email = if !committer_email.trim().is_empty() {
        committer_email.trim()
    } else {
        author_email.trim()
    };
    if email.is_empty() {
        return "unknown".into();
    }

    let resolved = author_resolve::resolve_author_email(db, email).await;
    let Some(user_id) = resolved.user_id.as_deref() else {
        return "unknown".into();
    };

    // Prefer primary email verified: user row must exist; noreply counts as forge-recognized.
    let Ok(user) = db.find_user_by_id(user_id).await else {
        return "unknown".into();
    };
    let Some(user) = user else {
        return "unknown".into();
    };
    let email_ok = user.email_verified_at.is_some()
        || author_resolve::is_forge_noreply_email(email);
    if !email_ok {
        return "unknown".into();
    }

    if signature_kind == "gpg" {
        let Ok(keys) = db.list_gpg_keys_for_user(user_id).await else {
            return "unknown".into();
        };
        let email_l = email.to_ascii_lowercase();
        let uid_ok = keys.iter().any(|k| {
            serde_json::from_str::<Vec<String>>(&k.uid_emails)
                .unwrap_or_default()
                .iter()
                .any(|u| u.eq_ignore_ascii_case(&email_l))
        });
        if !uid_ok {
            return "invalid".into();
        }
    }

    "valid".into()
}

#[allow(dead_code)]
pub fn allowed_signers_path(ring: &VerifyKeyring) -> Option<&Path> {
    ring.allowed_signers.as_deref()
}

#[allow(dead_code)]
pub fn gpg_home_path(ring: &VerifyKeyring) -> Option<&Path> {
    ring.gpg_home.as_deref()
}
