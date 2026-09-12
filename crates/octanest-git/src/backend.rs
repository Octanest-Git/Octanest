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
}

/// Async forge git operations. API/RPC never shell out directly.
#[async_trait::async_trait]
pub trait GitBackend: Send + Sync {
    /// Create a bare repository at `path` and point unborn `HEAD` at
    /// `refs/heads/{initial_branch}` (Git 2.5-compatible; no `--initial-branch`).
    async fn init_bare(&self, path: &Path, initial_branch: &str) -> Result<(), GitError>;
}
