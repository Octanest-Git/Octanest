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
}
