//! GIT-06 / D-28 Wave 0 stubs: default-branch soft-protect (rename/delete blocked).
//!
//! RED until branch RPCs + CliGitBackend land.

/// Owner cannot rename or delete the repository default branch (D-28).
#[tokio::test]
async fn repo_branch_soft_protect_blocks_default_rename_and_delete() {
    assert!(
        false,
        "Wave 0: rename/delete of default_branch must return a stable soft-protect error for the owner"
    );
}

/// Owner can create/rename/delete non-default branches (GIT-06 CRUD).
#[tokio::test]
async fn repo_branch_soft_protect_allows_non_default_crud() {
    assert!(
        false,
        "Wave 0: owner branch CRUD for non-default names must succeed once CliGitBackend lands"
    );
}
