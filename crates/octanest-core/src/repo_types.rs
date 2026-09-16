//! Repository name validation and DTOs (D-06). Separate from username rules.

use serde::{Deserialize, Serialize};

use crate::auth_types::is_reserved_username;
use crate::org_types::{CollaboratorPermission, OwnerType};

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
    /// Optional owner slug (username or org). Omit → session user (A5 / D-ORG-01).
    #[serde(default)]
    pub owner: Option<String>,
}

/// Public create-form defaults + catalog metadata (D-02–D-04, D-08).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCreateDefaults {
    pub default_visibility: RepoVisibility,
    pub stacks: Vec<RepoTemplateOption>,
    pub gitignores: Vec<RepoTemplateOption>,
}

/// Catalog option for stack / gitignore pickers (modal cards on `/new`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTemplateOption {
    pub id: String,
    pub label: String,
    pub group: String,
    /// Short human description shown in the picker modal.
    pub description: String,
    /// When set on a stack pack, `/new` auto-selects this gitignore and create
    /// seeds it unless the client sends an explicit gitignore (including `"none"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_gitignore: Option<String>,
}

/// Public repository metadata returned over RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoPublic {
    pub id: String,
    pub owner_id: String,
    /// Polymorphic owner: `user` | `org` (D-ORG-01).
    pub owner_type: OwnerType,
    /// Public slug label (username or org slug).
    pub owner_username: String,
    pub name: String,
    pub description: String,
    pub visibility: RepoVisibility,
    pub default_branch: String,
    pub updated_at: String,
    /// Caller has Admin capability (D-ORG-05 / settings UI).
    #[serde(default)]
    pub can_admin: bool,
    /// Caller has Write capability (D-ORG-05).
    #[serde(default)]
    pub can_write: bool,
}

/// `repo.listMine` — caller's non-deleted repos, recently updated first (GIT-01 / D-13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoListMineResponse {
    pub repos: Vec<RepoPublic>,
}

/// `repo.listByOwner` — repos under a user/org slug the caller can read (D-ORG-06 overview).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoListByOwnerRequest {
    pub owner: String,
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

/// `repo.commits` (log) input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCommitsRequest {
    pub owner: String,
    pub name: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    #[serde(default)]
    pub skip: u32,
    #[serde(default = "default_commits_limit")]
    pub limit: u32,
}

fn default_commits_limit() -> u32 {
    30
}

/// `repo.search` type discriminator (D-SRCH-14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoSearchType {
    Code,
    Commits,
    Issues,
    Pulls,
}

impl RepoSearchType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Commits => "commits",
            Self::Issues => "issues",
            Self::Pulls => "pulls",
        }
    }
}

fn default_search_limit() -> u32 {
    30
}

/// `repo.search` input (GIT-18 / D-SRCH-14).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSearchRequest {
    pub owner: String,
    pub name: String,
    #[serde(rename = "type")]
    pub search_type: RepoSearchType,
    #[serde(default)]
    pub q: String,
    /// Optional tree-ish; omit / empty → default branch (D-SRCH-06).
    #[serde(default, rename = "ref")]
    pub ref_name: Option<String>,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_search_limit")]
    pub limit: u32,
}

/// One hit in `repo.search` results (tagged by `kind`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RepoSearchHit {
    Code {
        path: String,
        line: u32,
        content: String,
    },
    Commit {
        sha: String,
        short_sha: String,
        subject: String,
        author_name: String,
        authored_at: String,
    },
    Issue {
        number: i64,
        title: String,
        state: String,
    },
    Pull {
        number: i64,
        title: String,
        state: String,
    },
}

/// `repo.search` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSearchResponse {
    #[serde(rename = "type")]
    pub search_type: RepoSearchType,
    pub q: String,
    pub hits: Vec<RepoSearchHit>,
    /// Soft cap / timeout truncated (D-SRCH-08).
    #[serde(default)]
    pub truncated: bool,
    pub offset: u32,
    pub limit: u32,
}

/// One commit row for history list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
    pub author_name: String,
    pub author_email: String,
    pub authored_at: String,
}

/// `repo.commits` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCommitsResponse {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub commits: Vec<RepoCommitSummary>,
    pub skip: u32,
    pub limit: u32,
}

/// `repo.commit` input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCommitRequest {
    pub owner: String,
    pub name: String,
    pub sha: String,
}

/// One file in a commit/compare diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoDiffFile {
    pub path: String,
    pub status: String,
    pub patch: String,
}

/// `repo.commit` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCommitResponse {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
    pub body: String,
    pub author_name: String,
    pub author_email: String,
    pub authored_at: String,
    pub parents: Vec<String>,
    pub files: Vec<RepoDiffFile>,
    pub truncated: bool,
}

/// `repo.compare` input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCompareRequest {
    pub owner: String,
    pub name: String,
    pub base: String,
    pub head: String,
}

/// `repo.compare` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCompareResponse {
    pub base: String,
    pub head: String,
    pub empty: bool,
    pub truncated: bool,
    pub files: Vec<RepoDiffFile>,
}

/// `repo.blame` input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBlameRequest {
    pub owner: String,
    pub name: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub path: String,
}

/// One blame line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBlameLine {
    pub sha: String,
    pub author_name: String,
    pub authored_at: String,
    pub line_number: u32,
    pub content: String,
}

/// `repo.blame` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBlameResponse {
    pub path: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub lines: Vec<RepoBlameLine>,
    pub truncated: bool,
}

/// `repo.branchCreate` input (GIT-06).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBranchCreateRequest {
    pub owner: String,
    pub name: String,
    pub branch: String,
    /// Start point (branch/tag/sha). Empty/omit → repository default branch.
    #[serde(default)]
    pub start: Option<String>,
}

/// `repo.branchRename` input (GIT-06).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBranchRenameRequest {
    pub owner: String,
    pub name: String,
    pub from: String,
    pub to: String,
}

/// `repo.branchDelete` input (GIT-06).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBranchDeleteRequest {
    pub owner: String,
    pub name: String,
    pub branch: String,
}

/// Branch mutate response — name of the resulting branch (create/rename) or deleted name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBranchMutationResponse {
    pub branch: String,
}

/// `repo.updateVisibility` input (D-26).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoUpdateVisibilityRequest {
    pub owner: String,
    pub name: String,
    pub visibility: RepoVisibility,
}

/// `repo.lfs.setEnabled` input — Admin-only per-repo LFS toggle (D-LFS-10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsSetEnabledRequest {
    pub owner: String,
    pub name: String,
    pub enabled: bool,
}

/// `repo.lfs.setEnabled` / `repo.lfs.getEnabled` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsEnabledResponse {
    pub enabled: bool,
}

/// `repo.lfs.getEnabled` input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsGetEnabledRequest {
    pub owner: String,
    pub name: String,
}

/// `repo.lfs.getStatus` — enable flag + light usage snapshot for Settings (D-LFS-16/19).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsStatusResponse {
    pub enabled: bool,
    pub object_count: i64,
    pub logical_bytes: i64,
}

/// Top / listed LFS object row for usage + browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsObjectEntry {
    pub oid: String,
    pub size: i64,
    pub refcount: i64,
}

/// `repo.lfs.getUsage` — this-repo breakdown (D-LFS-19).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsUsageResponse {
    pub enabled: bool,
    pub object_count: i64,
    pub logical_bytes: i64,
    pub quota_repo_bytes: i64,
    pub objects: Vec<RepoLfsObjectEntry>,
}

/// `repo.lfs.listObjects` input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsListObjectsRequest {
    pub owner: String,
    pub name: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// `repo.lfs.listObjects` response — in-app LFS browser (D-LFS-16).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsListObjectsResponse {
    pub enabled: bool,
    pub objects: Vec<RepoLfsObjectEntry>,
}

/// `repo.lfs.download` input — session Read path (D-LFS-18 / A2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsDownloadRequest {
    pub owner: String,
    pub name: String,
    pub oid: String,
}

/// Soft-capped base64 payload for browser Download (not git-lfs PAT path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLfsDownloadResponse {
    pub oid: String,
    pub size: i64,
    pub encoding: String,
    pub content: String,
}

/// Per-repo row in admin instance usage breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLfsRepoUsageEntry {
    pub repository_id: String,
    pub owner: String,
    pub name: String,
    pub object_count: i64,
    pub logical_bytes: i64,
}

/// Per-owner (user/org) row in admin instance usage breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLfsOwnerUsageEntry {
    pub owner_id: String,
    pub owner_slug: String,
    pub object_count: i64,
    pub logical_bytes: i64,
}

/// `admin.lfs.getUsage` — instance breakdown (D-LFS-19).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLfsUsageResponse {
    pub physical_bytes: i64,
    pub object_count: i64,
    pub logical_bytes: i64,
    pub by_repo: Vec<AdminLfsRepoUsageEntry>,
    pub by_owner: Vec<AdminLfsOwnerUsageEntry>,
}

/// `repo.softDelete` input — typed confirm name required (D-35 / T-07-24).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSoftDeleteRequest {
    pub owner: String,
    pub name: String,
    #[serde(rename = "confirmName", alias = "confirm_name")]
    pub confirm_name: String,
}

/// Soft-delete acknowledgement (DB row marked; disk purge deferred).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSoftDeleteResponse {
    pub name: String,
}

/// Effective instance LFS limits (Admin override or env default) — D-LFS-13.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLfsSettingsPublic {
    pub max_object_bytes: i64,
    pub quota_repo_bytes: i64,
    pub quota_user_bytes: i64,
    /// True when DB override is set for each field.
    pub max_object_bytes_overridden: bool,
    pub quota_repo_bytes_overridden: bool,
    pub quota_user_bytes_overridden: bool,
}

/// `admin.lfs.updateSettings` — null fields clear override (revert to env).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLfsUpdateSettingsRequest {
    #[serde(default)]
    pub max_object_bytes: Option<i64>,
    #[serde(default)]
    pub quota_repo_bytes: Option<i64>,
    #[serde(default)]
    pub quota_user_bytes: Option<i64>,
    /// When true, clear all overrides (use env defaults).
    #[serde(default)]
    pub clear_overrides: bool,
}
/// `repo.rename` input — Admin only; no type-confirm (D-REL-07 / GIT-16).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRenameRequest {
    pub owner: String,
    pub name: String,
    #[serde(rename = "newName", alias = "new_name")]
    pub new_name: String,
}

/// `repo.rename` response — updated public repo metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRenameResponse {
    pub repo: RepoPublic,
}

/// `repo.transfer` input — Admin only; type-confirm required (D-REL-09 / D-REL-10 / GIT-17).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTransferRequest {
    pub owner: String,
    pub name: String,
    #[serde(rename = "destOwner", alias = "dest_owner")]
    pub dest_owner: String,
    #[serde(rename = "destOwnerType", alias = "dest_owner_type")]
    pub dest_owner_type: OwnerType,
    #[serde(rename = "confirmName", alias = "confirm_name")]
    pub confirm_name: String,
}

/// `repo.transfer` response — updated public repo under the new owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTransferResponse {
    pub repo: RepoPublic,
}
/// Public collaborator row — no email (ORG-03 / D-ORG-02c).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCollaboratorPublic {
    pub user_id: String,
    pub username: String,
    pub permission: CollaboratorPermission,
    pub created_at: String,
}

/// `repo.collaborators.list` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCollaboratorsListResponse {
    pub collaborators: Vec<RepoCollaboratorPublic>,
}

/// `repo.collaborators.add` — existing instance user by username.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCollaboratorsAddRequest {
    pub owner: String,
    pub name: String,
    pub username: String,
    pub permission: CollaboratorPermission,
}

/// `repo.collaborators.update`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCollaboratorsUpdateRequest {
    pub owner: String,
    pub name: String,
    pub user_id: String,
    pub permission: CollaboratorPermission,
}

/// `repo.collaborators.remove`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCollaboratorsRemoveRequest {
    pub owner: String,
    pub name: String,
    pub user_id: String,
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

/// Sys-admin manual `git gc` (D-37). Omit owner+name to GC all active repos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGcRequest {
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Result of a manual or scheduled GC pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGcResponse {
    pub ok: bool,
    pub gc_count: u32,
    pub error_count: u32,
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
