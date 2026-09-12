//! `GitBackend` seam — all forge git ops go through this trait (GIT-09 / GIT-10 / D-32).
//!
//! Phase 7 ships only [`crate::cli::CliGitBackend`] (system `git` CLI ≥ 2.5).
//! A future `GixGitBackend` (gitoxide) should implement this same trait when
//! coverage reaches create/browse/branch/archive/gc parity — do not call `gix`
//! from API handlers.

use std::path::Path;

use thiserror::Error;

/// Errors from git backend operations (CLI or future gitoxide adapter).
#[derive(Debug, Error)]
pub enum GitError {
    #[error("git process failed: {0}")]
    Process(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid argument: {0}")]
    InvalidArg(String),
    /// Object/path/ref missing (empty repo, bad path, etc.).
    #[error("not found: {0}")]
    NotFound(String),
}

/// Kind of a tree entry (`git ls-tree` object type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeEntryKind {
    Blob,
    Tree,
    /// Gitlink / submodule (`160000`).
    Commit,
}

impl TreeEntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "blob" => Some(Self::Blob),
            "tree" => Some(Self::Tree),
            "commit" => Some(Self::Commit),
            _ => None,
        }
    }
}

/// One entry from `git ls-tree`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    pub mode: String,
    pub kind: TreeEntryKind,
    pub oid: String,
    pub name: String,
}

/// A ref from `git for-each-ref` / `show-ref`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRef {
    pub name: String,
    pub oid: String,
}

/// One commit from `git log` (paged history).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
    pub author_name: String,
    pub author_email: String,
    /// Author date as ISO-8601 (`%aI`).
    pub authored_at: String,
}

/// One file in a commit or compare diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFile {
    pub path: String,
    /// `added` | `modified` | `deleted` | `renamed` | `copied` | `unknown`
    pub status: String,
    /// Unified diff hunk text (may be truncated by soft caps).
    pub patch: String,
}

/// Full commit detail for `/commit/{sha}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDetail {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
    pub body: String,
    pub author_name: String,
    pub author_email: String,
    pub authored_at: String,
    pub parents: Vec<String>,
    pub files: Vec<DiffFile>,
    /// True when patch payload was soft-capped (D-20 / T-07-18).
    pub truncated: bool,
}

/// Compare `base...head` (or empty when identical).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    pub base: String,
    pub head: String,
    pub files: Vec<DiffFile>,
    pub empty: bool,
    pub truncated: bool,
}

/// One blame line (text files).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameLine {
    pub sha: String,
    pub author_name: String,
    pub authored_at: String,
    pub line_number: u32,
    pub content: String,
}

/// Blame for a path at a ref.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameFile {
    pub path: String,
    pub ref_name: String,
    pub lines: Vec<BlameLine>,
    pub truncated: bool,
}

/// Soft cap for unified patch bytes in show/diff responses (D-20 / T-07-18).
pub const DIFF_SOFT_MAX_BYTES: usize = 1_048_576;
/// Soft cap for blame line count (D-20).
pub const BLAME_SOFT_MAX_LINES: usize = 10_000;

/// Soft timeout for `git archive` (T-07-22).
pub const ARCHIVE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Source archive format for [`GitBackend::archive`] (GIT-07 / D-29).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
}

impl ArchiveFormat {
    pub fn as_git_format(self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::TarGz => "tar.gz",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Zip => "application/zip",
            Self::TarGz => "application/gzip",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::TarGz => "tar.gz",
        }
    }
}

/// Async forge git operations. API/RPC never shell out directly.
#[async_trait::async_trait]
pub trait GitBackend: Send + Sync {
    /// Create a bare repository at `path` and point unborn `HEAD` at
    /// `refs/heads/{initial_branch}` (Git 2.5-compatible; no `--initial-branch`).
    async fn init_bare(&self, path: &Path, initial_branch: &str) -> Result<(), GitError>;

    /// Write `files` (relative path → bytes) as a single commit on `branch` in an
    /// existing bare repo (temp worktree + push). No-op when `files` is empty.
    async fn seed_commit(
        &self,
        bare_path: &Path,
        branch: &str,
        message: &str,
        files: &[(String, Vec<u8>)],
    ) -> Result<(), GitError>;

    /// List tree entries at `path` under `treeish` (branch/tag/sha). Empty repo /
    /// unborn HEAD → `Ok(vec![])` (not an error).
    async fn ls_tree(
        &self,
        repo: &Path,
        treeish: &str,
        path: &str,
    ) -> Result<Vec<TreeEntry>, GitError>;

    /// Read blob bytes at `path` for `treeish`. Missing path → [`GitError::NotFound`].
    async fn cat_blob(
        &self,
        repo: &Path,
        treeish: &str,
        path: &str,
    ) -> Result<Vec<u8>, GitError>;

    /// List refs under `refs/heads` and `refs/tags` (name + oid). Empty → `Ok(vec![])`.
    async fn list_refs(&self, repo: &Path) -> Result<Vec<GitRef>, GitError>;

    /// Paged `git log` for `refname` (`skip` / `limit`). Empty history → `Ok(vec![])`.
    async fn log(
        &self,
        repo: &Path,
        refname: &str,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<CommitSummary>, GitError>;

    /// Commit metadata + per-file unified patches (`git show`).
    async fn show_commit(&self, repo: &Path, sha: &str) -> Result<CommitDetail, GitError>;

    /// Unified diff `base...head`. Identical trees → `empty: true` (not an error).
    async fn diff(
        &self,
        repo: &Path,
        base: &str,
        head: &str,
    ) -> Result<DiffResult, GitError>;

    /// Per-line blame for a text file (`git blame --line-porcelain`).
    async fn blame(
        &self,
        repo: &Path,
        refname: &str,
        path: &str,
    ) -> Result<BlameFile, GitError>;

    /// Create branch `name` pointing at `start` (branch/tag/sha).
    async fn branch_create(
        &self,
        repo: &Path,
        name: &str,
        start: &str,
    ) -> Result<(), GitError>;

    /// Rename local branch `from` → `to`.
    async fn branch_rename(&self, repo: &Path, from: &str, to: &str) -> Result<(), GitError>;

    /// Delete local branch `name` (`git branch -D`).
    async fn branch_delete(&self, repo: &Path, name: &str) -> Result<(), GitError>;

    /// Build a source archive (`git archive`) for `treeish` with `--prefix={prefix}/`.
    /// Empty / unborn refs → [`GitError::NotFound`] (not a panic).
    async fn archive(
        &self,
        repo: &Path,
        treeish: &str,
        format: ArchiveFormat,
        prefix: &str,
    ) -> Result<Vec<u8>, GitError>;

    /// Run `git gc` on a bare (or worktree) repository (D-37).
    async fn gc(&self, repo: &Path) -> Result<(), GitError>;
}
