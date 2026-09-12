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

/// Create-repository input (RPC wired in 07-12; templates in 07-03).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// When omitted, API uses instance `default_visibility` (else public) — D-08.
    #[serde(default)]
    pub visibility: Option<RepoVisibility>,
    /// Stack preset pack id under `assets/stack-presets/` (omit / null = none).
    #[serde(default)]
    pub stack_id: Option<String>,
    /// SPDX license id or omit / null / `"none"` for no LICENSE file.
    #[serde(default)]
    pub license_id: Option<String>,
    /// Gitignore catalog id under `assets/gitignore/` (omit / null / `"none"` = none).
    #[serde(default)]
    pub gitignore_id: Option<String>,
}

/// Public create-form defaults + catalog metadata (D-02–D-04, D-08).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCreateDefaults {
    pub default_visibility: RepoVisibility,
    pub stacks: Vec<RepoTemplateOption>,
    pub gitignores: Vec<RepoTemplateOption>,
}

/// Select option for stack / gitignore pickers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTemplateOption {
    pub id: String,
    pub label: String,
    pub group: String,
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

/// `repo.listMine` — caller's non-deleted repos, recently updated first (GIT-01 / D-13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoListMineResponse {
    pub repos: Vec<RepoPublic>,
}

/// `repo.get` / `repo.refs` input — owner + name (GIT-05).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGetRequest {
    pub owner: String,
    pub name: String,
}

/// `repo.tree` input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTreeRequest {
    pub owner: String,
    pub name: String,
    /// Branch/tag/sha. Serialized as `ref` for API ergonomics.
    #[serde(rename = "ref")]
    pub ref_name: String,
    #[serde(default)]
    pub path: Option<String>,
}

/// One `ls-tree` entry for RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTreeEntry {
    pub mode: String,
    pub kind: String,
    pub oid: String,
    pub name: String,
}

/// `repo.tree` response — empty repo sets `empty: true` without 500.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTreeResponse {
    pub empty: bool,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub path: String,
    pub entries: Vec<RepoTreeEntry>,
}

/// `repo.blob` input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBlobRequest {
    pub owner: String,
    pub name: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub path: String,
}

/// Soft-capped blob payload for UI (D-20).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBlobResponse {
    pub path: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub size: u64,
    pub truncated: bool,
    pub is_binary: bool,
    pub encoding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub soft_max_bytes: u64,
}

/// One ref from `repo.refs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRefEntry {
    pub name: String,
    pub oid: String,
}

/// `repo.refs` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRefsResponse {
    pub refs: Vec<RepoRefEntry>,
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
