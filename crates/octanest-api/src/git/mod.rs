//! API-side git helpers: path layout under `OCTANEST_REPOS_DIR` (D-30 / D-31).
//!
//! All process invocation lives in `octanest-git` (`CliGitBackend`). This module
//! only builds safe on-disk paths under the configured repos root.

use std::path::{Path, PathBuf};

use octanest_core::AppError;

/// Bare repo path: `{repos_dir}/{owner}/{name}.git` (D-30).
///
/// `owner` and `name` must already be validated (username / `validate_repo_name`).
pub fn bare_repo_path(repos_dir: &Path, owner: &str, name: &str) -> Result<PathBuf, AppError> {
    if owner.is_empty()
        || name.is_empty()
        || owner.contains('/')
        || owner.contains('\\')
        || owner.contains("..")
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
    {
        return Err(AppError::new(
            "repo.invalid_path",
            "invalid repository path components",
        ));
    }
    Ok(repos_dir.join(owner).join(format!("{name}.git")))
}
