//! Repository DTOs and name validation (D-06). Separate from username rules.

use serde::{Deserialize, Serialize};

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
    pub visibility: RepoVisibility,
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

/// GitHub-ish repo name rules (D-06): letters, digits, hyphen, underscore, period;
/// reject empty and reserved path segments. Stub until GREEN implements.
pub fn validate_repo_name(_raw: &str) -> Result<(), String> {
    Err("repository name validation not implemented".into())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            err.contains("reserved") || err.contains("not implemented"),
            "expected reserved rejection, got: {err}"
        );
        // After GREEN: reserved path segments must be rejected for repo names too.
        let _ = is_reserved_username("login");
    }
}
