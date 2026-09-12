//! Repository name validation and DTOs (D-06). Separate from username rules.

use serde::{Deserialize, Serialize};

use crate::auth_types::is_reserved_username;

/// Repo visibility. Serialized lowercase: `public` | `private`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoVisibility {
    Public,
    Private,
}

impl RepoVisibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            other => Err(format!("invalid visibility: {other}")),
        }
    }
}

/// Create-repository input (RPC wired in 07-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// When omitted, API uses instance `default_visibility` (else public) — D-08.
    #[serde(default)]
    pub visibility: Option<RepoVisibility>,
}

/// Public repository metadata returned over RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoPublic {
    pub id: String,
    pub owner_id: String,
    pub owner_username: String,
    pub name: String,
    pub description: String,
    pub visibility: RepoVisibility,
    pub default_branch: String,
    pub updated_at: String,
}

/// GitHub-ish repo name rules (D-06): 1–100 chars, ascii letters/digits/hyphen/underscore/period;
/// no leading/trailing `.` or `-`; not `.` / `..`; not a reserved path segment.
pub fn validate_repo_name(raw: &str) -> Result<(), String> {
    let name = raw.trim();
    if name.is_empty() || name.len() > 100 {
        return Err("repository name must be 1–100 characters".into());
    }
    if name == "." || name == ".." {
        return Err("repository name is invalid".into());
    }
    if name.starts_with('-')
        || name.ends_with('-')
        || name.starts_with('.')
        || name.ends_with('.')
    {
        return Err("repository name cannot start or end with a hyphen or period".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("repository name must be alphanumeric, hyphen, underscore, or period".into());
    }
    if is_reserved_username(name) {
        return Err("repository name is reserved".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_types::is_reserved_username;

    #[test]
    fn validate_repo_name_accepts_my_app() {
        assert!(
            validate_repo_name("my_app").is_ok(),
            "D-06: underscore must be allowed in repo names"
        );
    }

    #[test]
    fn validate_repo_name_accepts_dotted_name() {
        assert!(validate_repo_name("my.app").is_ok());
    }

    #[test]
    fn validate_repo_name_rejects_empty() {
        assert!(validate_repo_name("").is_err());
        assert!(validate_repo_name("   ").is_err());
    }

    #[test]
    fn validate_repo_name_rejects_reserved() {
        let err = validate_repo_name("login").unwrap_err();
        assert!(
            err.contains("reserved"),
            "expected reserved rejection, got: {err}"
        );
        assert!(is_reserved_username("login"));
    }
}
