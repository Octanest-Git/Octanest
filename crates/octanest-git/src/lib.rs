//! Octanest git forge backend.
//!
//! Phase 7 ships [`CliGitBackend`] via the system `git` CLI (2.5+).
//! A future `GixGitBackend` (gitoxide) is intentionally deferred — keep the
//! [`GitBackend`] abstraction as the swap seam (GIT-09 / GIT-10 / D-32).
//!
//! Boot version gate (`assert_git_version`) greens in plan 07-17.

pub mod backend;
pub mod cli;
pub mod version;

pub use backend::{GitBackend, GitError, GitRef, TreeEntry, TreeEntryKind};
pub use cli::CliGitBackend;
pub use version::{assert_git_version, parse_git_version};

pub fn crate_name() -> &'static str {
    "octanest-git"
}
