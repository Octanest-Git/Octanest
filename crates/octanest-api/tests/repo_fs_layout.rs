//! GIT-08 / D-30 Wave 0 stubs: bare repo path under repos_dir.
//!
//! RED until create wires `{repos_dir}/{owner}/{name}.git`.

use std::path::PathBuf;

/// After verified create, bare git dir exists at `{repos_dir}/{owner}/{name}.git`.
#[tokio::test]
async fn repo_fs_layout_bare_path_under_repos_dir() {
    let repos_dir = PathBuf::from("/tmp/octanest-wave0-repos-placeholder");
    let owner = "owner1";
    let name = "hello-world";
    let expected = repos_dir.join(owner).join(format!("{name}.git"));

    assert!(
        expected.is_dir(),
        "Wave 0: bare repo must exist at {{repos_dir}}/{{owner}}/{{name}}.git — missing {}",
        expected.display()
    );
}
