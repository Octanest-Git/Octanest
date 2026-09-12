//! Octanest git forge backend.
//!
//! Phase 7 ships [`CliGitBackend`](crate) via the system `git` CLI (2.5+).
//! A future `GixGitBackend` (gitoxide) is intentionally deferred — keep the
//! `GitBackend` abstraction as the swap seam (GIT-09 / GIT-10 / D-32).
//!
//! Wave 0: version gate + archive format stubs only. Full trait + CLI runner
//! land in later plans.

pub mod version;

pub use version::{assert_git_version, parse_git_version};

pub fn crate_name() -> &'static str {
    "octanest-git"
}
