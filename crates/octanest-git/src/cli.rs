//! CLI-backed [`GitBackend`] via `tokio::process::Command` argv arrays (never `sh -c`).

use std::path::{Component, Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::backend::{GitBackend, GitError, GitRef, TreeEntry, TreeEntryKind};

/// System `git` CLI adapter (D-32). Only backend registered in Phase 7.
#[derive(Debug, Default, Clone)]
pub struct CliGitBackend;

impl CliGitBackend {
    pub fn new() -> Self {
        Self
    }
}

async fn run_git(args: &[&str]) -> Result<(), GitError> {
    let _ = run_git_stdout(args).await?;
    Ok(())
}

async fn run_git_stdout(args: &[&str]) -> Result<Vec<u8>, GitError> {
    let output = Command::new("git")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;

    if output.status.success() {
        return Ok(output.stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(GitError::Process(format!(
        "git {} failed (status {:?}): {}{}",
        args.join(" "),
        output.status.code(),
        stderr.trim(),
        if stdout.trim().is_empty() {
            String::new()
        } else {
            format!(" | {}", stdout.trim())
        }
    )))
}

/// Reject NUL / `..` / absolute-looking refs (T-07-15).
fn validate_treeish(treeish: &str) -> Result<&str, GitError> {
    let t = treeish.trim();
    if t.is_empty() || t.contains('\0') || t.contains("..") {
        return Err(GitError::InvalidArg(format!("invalid treeish: {treeish}")));
    }
    Ok(t)
}

/// Reject path traversal in tree/blob paths (T-07-15). Empty path = repo root.
fn validate_repo_rel_path(path: &str) -> Result<String, GitError> {
    let rel = path.trim().trim_start_matches('/');
    if rel.contains('\0') {
        return Err(GitError::InvalidArg("path contains NUL".into()));
    }
    if rel.is_empty() {
        return Ok(String::new());
    }
    let candidate = Path::new(rel);
    if candidate.is_absolute()
        || candidate.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(GitError::InvalidArg(format!("path escapes repo: {path}")));
    }
    Ok(rel.to_string())
}

fn parse_ls_tree_line(line: &str) -> Option<TreeEntry> {
    // <mode> SP <type> SP <object> TAB <file>
    let (meta, name) = line.split_once('\t')?;
    let mut parts = meta.split_whitespace();
    let mode = parts.next()?.to_string();
    let kind = TreeEntryKind::parse(parts.next()?)?;
    let oid = parts.next()?.to_string();
    if name.is_empty() || name.contains('\0') {
        return None;
    }
    Some(TreeEntry {
        mode,
        kind,
        oid,
        name: name.to_string(),
    })
}

#[async_trait::async_trait]
impl GitBackend for CliGitBackend {
    async fn init_bare(&self, path: &Path, initial_branch: &str) -> Result<(), GitError> {
        let branch = initial_branch.trim();
        if branch.is_empty()
            || branch.contains('/')
            || branch.contains('\0')
            || branch.contains("..")
        {
            return Err(GitError::InvalidArg(format!(
                "invalid initial branch: {initial_branch}"
            )));
        }

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let path_str = path.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 repo path: {}", path.display()))
        })?;

        // Pattern 2 (RESEARCH): init --bare then symbolic-ref (no `--initial-branch`; Git 2.5+).
        run_git(&["init", "--bare", path_str]).await?;
        let head_ref = format!("refs/heads/{branch}");
        run_git(&["-C", path_str, "symbolic-ref", "HEAD", &head_ref]).await?;
        Ok(())
    }

    async fn seed_commit(
        &self,
        bare_path: &Path,
        branch: &str,
        message: &str,
        files: &[(String, Vec<u8>)],
    ) -> Result<(), GitError> {
        if files.is_empty() {
            return Ok(());
        }

        let branch = branch.trim();
        if branch.is_empty()
            || branch.contains('/')
            || branch.contains('\0')
            || branch.contains("..")
        {
            return Err(GitError::InvalidArg(format!(
                "invalid branch for seed: {branch}"
            )));
        }
        if message.contains('\0') {
            return Err(GitError::InvalidArg("commit message contains NUL".into()));
        }

        let bare_str = bare_path.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 bare path: {}", bare_path.display()))
        })?;

        let tmp = tempfile::tempdir().map_err(GitError::Io)?;
        let work = tmp.path();
        let work_str = work.to_str().ok_or_else(|| {
            GitError::InvalidArg("non-utf8 temp worktree path".into())
        })?;

        run_git(&["init", work_str]).await?;
        // Detached orphan-style first commit on the target branch name.
        run_git(&["-C", work_str, "symbolic-ref", "HEAD", &format!("refs/heads/{branch}")])
            .await?;

        for (rel, content) in files {
            let dest = safe_worktree_path(work, rel)?;
            if let Some(parent) = dest.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&dest, content).await?;
        }

        run_git(&["-C", work_str, "config", "user.email", "noreply@octanest.local"]).await?;
        run_git(&["-C", work_str, "config", "user.name", "Octanest"]).await?;
        run_git(&["-C", work_str, "add", "-A"]).await?;
        run_git(&["-C", work_str, "commit", "-m", message]).await?;
        run_git(&["-C", work_str, "remote", "add", "origin", bare_str]).await?;
        let refspec = format!("HEAD:refs/heads/{branch}");
        run_git(&["-C", work_str, "push", "origin", &refspec]).await?;
        Ok(())
    }

    async fn ls_tree(
        &self,
        repo: &Path,
        treeish: &str,
        path: &str,
    ) -> Result<Vec<TreeEntry>, GitError> {
        let treeish = validate_treeish(treeish)?;
        let path = validate_repo_rel_path(path)?;
        let repo_str = repo.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 repo path: {}", repo.display()))
        })?;

        // Unborn HEAD / empty repo: rev-parse fails → structured empty.
        let rev = Command::new("git")
            .args(["-C", repo_str, "rev-parse", "--verify", &format!("{treeish}^{{commit}}")])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
        if !rev.status.success() {
            return Ok(Vec::new());
        }

        let mut args: Vec<String> = vec![
            "-C".into(),
            repo_str.into(),
            "ls-tree".into(),
        ];
        if path.is_empty() {
            args.push(treeish.to_string());
        } else {
            // List children of the path tree (not the tree entry itself).
            args.push(format!("{treeish}:{path}"));
        }

        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let stdout = match run_git_stdout(&arg_refs).await {
            Ok(b) => b,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("Not a valid object name")
                    || msg.contains("does not exist")
                    || msg.contains("not exist")
                    || msg.contains("bad file")
                {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        };
        let text = String::from_utf8_lossy(&stdout);
        let mut entries = Vec::new();
        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            if let Some(entry) = parse_ls_tree_line(line) {
                // When listing a subdirectory, git prints the basename only if
                // we pass path/; with path as tree, entries are children.
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    async fn cat_blob(
        &self,
        repo: &Path,
        treeish: &str,
        path: &str,
    ) -> Result<Vec<u8>, GitError> {
        let treeish = validate_treeish(treeish)?;
        let path = validate_repo_rel_path(path)?;
        if path.is_empty() {
            return Err(GitError::InvalidArg("blob path required".into()));
        }
        let repo_str = repo.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 repo path: {}", repo.display()))
        })?;

        let spec = format!("{treeish}:{path}");
        let output = Command::new("git")
            .args(["-C", repo_str, "cat-file", "-p", &spec])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;

        if output.status.success() {
            return Ok(output.stdout);
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Not a valid object name")
            || stderr.contains("does not exist")
            || stderr.contains("exists on disk, but not in")
            || stderr.contains("bad file")
        {
            return Err(GitError::NotFound(format!("blob {spec}")));
        }
        Err(GitError::Process(format!(
            "git cat-file failed: {}",
            stderr.trim()
        )))
    }

    async fn list_refs(&self, repo: &Path) -> Result<Vec<GitRef>, GitError> {
        let repo_str = repo.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 repo path: {}", repo.display()))
        })?;

        let output = Command::new("git")
            .args([
                "-C",
                repo_str,
                "for-each-ref",
                "--format=%(objectname) %(refname)",
                "refs/heads",
                "refs/tags",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Empty bare repo may still succeed with empty stdout.
            if output.stdout.is_empty() {
                return Ok(Vec::new());
            }
            return Err(GitError::Process(format!(
                "git for-each-ref failed: {}",
                stderr.trim()
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut refs = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((oid, name)) = line.split_once(' ') else {
                continue;
            };
            refs.push(GitRef {
                name: name.to_string(),
                oid: oid.to_string(),
            });
        }
        Ok(refs)
    }
}

/// Reject absolute paths and `..` components (T-07-09).
fn safe_worktree_path(work: &Path, rel: &str) -> Result<PathBuf, GitError> {
    let rel = rel.trim_start_matches('/');
    if rel.is_empty() || rel.contains('\0') {
        return Err(GitError::InvalidArg(format!("invalid seed path: {rel}")));
    }
    let candidate = Path::new(rel);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(GitError::InvalidArg(format!(
            "path escapes worktree: {rel}"
        )));
    }
    let dest = work.join(candidate);
    let work_canon = work;
    if !dest.starts_with(work_canon) {
        return Err(GitError::InvalidArg(format!(
            "path escapes worktree: {rel}"
        )));
    }
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{GitBackend, TreeEntryKind};

    #[tokio::test]
    async fn ls_tree_returns_dirs_files_and_gitlink_modes() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("modes.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "seed",
            &[
                ("README.md".into(), b"# hi\n".to_vec()),
                ("src/lib.rs".into(), b"fn x() {}\n".to_vec()),
            ],
        )
        .await
        .unwrap();

        // Add gitlink via plumbing on a worktree push.
        let wt = tempfile::tempdir().unwrap();
        let wt_s = wt.path().to_str().unwrap();
        let bare_s = bare.to_str().unwrap();
        run_git(&["clone", bare_s, wt_s]).await.unwrap();
        let index_info = b"160000 commit 0123456789abcdef0123456789abcdef01234567\tvendor/dep\n";
        let mut child = Command::new("git")
            .args(["-C", wt_s, "update-index", "--add", "--index-info"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        use tokio::io::AsyncWriteExt;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(index_info)
            .await
            .unwrap();
        let out = child.wait_with_output().await.unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        run_git(&["-C", wt_s, "commit", "-m", "gitlink"]).await.unwrap();
        run_git(&["-C", wt_s, "push", "origin", "HEAD:main"]).await.unwrap();

        let entries = git.ls_tree(&bare, "main", "").await.expect("ls_tree root");
        let readme = entries.iter().find(|e| e.name == "README.md").expect("blob");
        assert_eq!(readme.mode, "100644");
        assert_eq!(readme.kind, TreeEntryKind::Blob);
        let src = entries.iter().find(|e| e.name == "src").expect("tree");
        assert_eq!(src.mode, "040000");
        assert_eq!(src.kind, TreeEntryKind::Tree);
        let vendor = entries.iter().find(|e| e.name == "vendor").expect("vendor");
        assert_eq!(vendor.kind, TreeEntryKind::Tree);

        let nested = git.ls_tree(&bare, "main", "vendor").await.expect("vendor");
        let dep = nested.iter().find(|e| e.name == "dep").expect("gitlink");
        assert_eq!(dep.mode, "160000");
        assert_eq!(dep.kind, TreeEntryKind::Commit);
    }

    #[tokio::test]
    async fn cat_blob_returns_seeded_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("blob.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "c",
            &[("a.txt".into(), b"hello-blob\n".to_vec())],
        )
        .await
        .unwrap();
        let bytes = git.cat_blob(&bare, "main", "a.txt").await.unwrap();
        assert_eq!(bytes, b"hello-blob\n");
    }

    #[tokio::test]
    async fn ls_tree_empty_repo_returns_empty_vec() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("empty.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        let entries = git.ls_tree(&bare, "main", "").await.unwrap();
        assert!(entries.is_empty());
    }
}
