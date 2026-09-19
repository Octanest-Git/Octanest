//! CLI-backed [`GitBackend`] via `tokio::process::Command` argv arrays (never `sh -c`).

use std::path::{Component, Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::backend::{
    ArchiveFormat, BlameFile, BlameLine, CommitDetail, CommitSummary, ContributorSummary, DiffFile,
    DiffResult, GitBackend, GitError, GitRef, GrepHit, GrepResult, SizedBlobEntry, TreeEntry,
    TreeEntryKind, ARCHIVE_TIMEOUT, BLAME_SOFT_MAX_LINES, DIFF_SOFT_MAX_BYTES,
};

/// System `git` CLI adapter (D-32). Only backend registered in Phase 7.
#[derive(Debug, Default, Clone)]
pub struct CliGitBackend;

impl CliGitBackend {
    pub fn new() -> Self {
        Self
    }
}

/// Install bare-repo `hooks/update` for branch protection (Phase 13 / D-19).
/// Idempotent — overwrites with the known-good script.
pub async fn install_protection_hooks(bare: &Path) -> Result<(), GitError> {
    let hooks = bare.join("hooks");
    tokio::fs::create_dir_all(&hooks).await?;
    let update = hooks.join("update");
    // D-PKG-02: fail-closed when helper missing in production|cloud
    // (mirrors webhook deliver.rs env signal); fail-open for compose/dev.
    let script = r#"#!/bin/sh
# Octanest branch protection update hook (Phase 13 / D-19; D-PKG-02)
refname="$1"
oldrev="$2"
newrev="$3"
helper="${OCTANEST_PROTECTION_HELPER:-}"
if [ -z "$helper" ] || [ ! -x "$helper" ]; then
  env_name="${OCTANEST_ENV:-development}"
  case "$env_name" in
    production|cloud)
      echo "octanest: protection helper missing or not executable (OCTANEST_ENV=$env_name)" >&2
      exit 1
      ;;
    *)
      exit 0
      ;;
  esac
fi
exec "$helper" update "$refname" "$oldrev" "$newrev"
"#;
    tokio::fs::write(&update, script).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&update).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&update, perms).await?;
    }
    Ok(())
}

/// Ensure protection hooks exist (lazy reconcile).
pub async fn reconcile_protection_hooks(bare: &Path) -> Result<(), GitError> {
    let update = bare.join("hooks").join("update");
    if tokio::fs::metadata(&update).await.is_ok() {
        return Ok(());
    }
    install_protection_hooks(bare).await
}

async fn run_git(args: &[&str]) -> Result<(), GitError> {
    let _ = run_git_stdout(args).await?;
    Ok(())
}

async fn run_git_stdout(args: &[&str]) -> Result<Vec<u8>, GitError> {
    // Tests / CI often have no global git identity; env overrides avoid
    // "Author identity unknown" without mutating the runner's ~/.gitconfig.
    let output = Command::new("git")
        .args(args)
        .env("GIT_AUTHOR_NAME", "Octanest")
        .env("GIT_AUTHOR_EMAIL", "noreply@octanest.local")
        .env("GIT_COMMITTER_NAME", "Octanest")
        .env("GIT_COMMITTER_EMAIL", "noreply@octanest.local")
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

/// Reject NUL / `..` / leading `-` / absolute-looking refs (T-07-15 / T-07-17 / CR-02).
fn validate_treeish(treeish: &str) -> Result<&str, GitError> {
    let t = treeish.trim();
    if t.is_empty() || t.contains('\0') || t.contains("..") || t.starts_with('-') {
        return Err(GitError::InvalidArg(format!("invalid treeish: {treeish}")));
    }
    // Allow branch/tag/sha characters; reject shell metacharacters.
    if t.chars().any(|c| {
        matches!(
            c,
            ';' | '|' | '&' | '`' | '$' | '(' | ')' | '<' | '>' | '\n' | '\r' | ' '
        )
    }) {
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

fn repo_str(repo: &Path) -> Result<&str, GitError> {
    repo.to_str()
        .ok_or_else(|| GitError::InvalidArg(format!("non-utf8 repo path: {}", repo.display())))
}

fn parse_commit_summary_record(record: &str) -> Option<CommitSummary> {
    let parts: Vec<&str> = record.split('\0').collect();
    if parts.len() < 6 {
        return None;
    }
    let sha = parts[0].trim();
    if sha.is_empty() {
        return None;
    }
    Some(CommitSummary {
        sha: sha.to_string(),
        short_sha: parts[1].trim().to_string(),
        subject: parts[2].to_string(),
        author_name: parts[3].to_string(),
        author_email: parts[4].to_string(),
        authored_at: parts[5].trim().to_string(),
    })
}

fn status_from_diff_header(header: &str) -> String {
    if header.contains("new file mode") {
        "added".into()
    } else if header.contains("deleted file mode") {
        "deleted".into()
    } else if header.contains("rename from") || header.contains("similarity index") {
        "renamed".into()
    } else if header.contains("copy from") {
        "copied".into()
    } else {
        "modified".into()
    }
}

fn path_from_diff_git_line(line: &str) -> String {
    // diff --git a/path b/path  (paths may include spaces when quoted)
    let rest = line.strip_prefix("diff --git ").unwrap_or(line);
    let mut parts = rest.split_whitespace();
    let _a = parts.next().unwrap_or("");
    let b = parts.next().unwrap_or("");
    let path = b.strip_prefix("b/").unwrap_or(b);
    if path.starts_with('"') {
        path.trim_matches('"').to_string()
    } else {
        path.to_string()
    }
}

/// Split unified `git show` / `git diff` patch output into per-file hunks.
fn parse_unified_diff_files(patch: &str, soft_max: usize) -> (Vec<DiffFile>, bool) {
    let mut files = Vec::new();
    let mut truncated = false;
    let mut total = 0usize;
    let mut current_header = String::new();
    let mut current_body = String::new();
    let mut current_path = String::new();

    let flush = |header: &str,
                 body: &str,
                 path: &str,
                 files: &mut Vec<DiffFile>,
                 total: &mut usize,
                 truncated: &mut bool| {
        if path.is_empty() && header.is_empty() && body.is_empty() {
            return;
        }
        let chunk = if header.is_empty() {
            body.to_string()
        } else if body.is_empty() {
            header.to_string()
        } else {
            format!("{header}{body}")
        };
        let take = if *total >= soft_max {
            *truncated = true;
            String::new()
        } else if *total + chunk.len() > soft_max {
            *truncated = true;
            let remain = soft_max - *total;
            chunk.chars().take(remain).collect()
        } else {
            chunk
        };
        *total += take.len();
        if path.is_empty() && take.is_empty() {
            return;
        }
        files.push(DiffFile {
            path: if path.is_empty() {
                "(unknown)".into()
            } else {
                path.to_string()
            },
            status: status_from_diff_header(header),
            patch: take,
        });
    };

    for line in patch.lines() {
        if line.starts_with("diff --git ") {
            flush(
                &current_header,
                &current_body,
                &current_path,
                &mut files,
                &mut total,
                &mut truncated,
            );
            current_path = path_from_diff_git_line(line);
            current_header = format!("{line}\n");
            current_body.clear();
        } else if current_path.is_empty() && current_header.is_empty() {
            // Skip commit metadata preamble before first diff.
            continue;
        } else if line.starts_with("@@")
            || line.starts_with('+')
            || line.starts_with('-')
            || line.starts_with(' ')
            || line == "\\ No newline at end of file"
        {
            current_body.push_str(line);
            current_body.push('\n');
        } else {
            current_header.push_str(line);
            current_header.push('\n');
        }
        if truncated {
            break;
        }
    }
    flush(
        &current_header,
        &current_body,
        &current_path,
        &mut files,
        &mut total,
        &mut truncated,
    );
    (files, truncated)
}

fn parse_blame_porcelain(text: &str, soft_max_lines: usize) -> (Vec<BlameLine>, bool) {
    let mut lines = Vec::new();
    let mut truncated = false;
    let mut sha = String::new();
    let mut author = String::new();
    let mut authored_at = String::new();
    let mut line_number = 0u32;

    for raw in text.lines() {
        if raw.starts_with('\t') {
            if lines.len() >= soft_max_lines {
                truncated = true;
                break;
            }
            lines.push(BlameLine {
                sha: sha.clone(),
                author_name: author.clone(),
                authored_at: authored_at.clone(),
                line_number,
                content: raw[1..].to_string(),
            });
            continue;
        }
        if raw.is_empty() {
            continue;
        }
        let mut parts = raw.split_whitespace();
        let first = parts.next().unwrap_or("");
        if first.len() >= 40 && first.chars().all(|c| c.is_ascii_hexdigit()) {
            sha = first.to_string();
            let _orig = parts.next();
            if let Some(final_no) = parts.next() {
                line_number = final_no.parse().unwrap_or(0);
            }
        } else if first == "author" {
            author = raw.strip_prefix("author ").unwrap_or("").to_string();
        } else if first == "author-time" {
            let secs: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            authored_at = chrono_like_iso(secs);
        }
    }
    (lines, truncated)
}

fn chrono_like_iso(secs: i64) -> String {
    if secs < 0 {
        return String::new();
    }
    // Howard Hinnant civil-from-days (proleptic Gregorian), UTC.
    let z = (secs / 86_400) + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let tod = (secs % 86_400) as u32;
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;
    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
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
        // Phase 13 / D-19: install branch-protection update hook (reconcile-safe).
        install_protection_hooks(path).await?;
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

        // Relative bare paths (e.g. default OCTANEST_REPOS_DIR=var/repos) must be
        // absolutized: push runs with `-C` in a temp worktree, so git would otherwise
        // resolve `origin` relative to /tmp/... and fail with "does not appear to be a
        // git repository".
        let bare_abs = absolute_path(bare_path)?;
        let bare_str = bare_abs.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 bare path: {}", bare_abs.display()))
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

    // GREEN: real git log / show / diff / blame via argv.
    async fn log(
        &self,
        repo: &Path,
        refname: &str,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<CommitSummary>, GitError> {
        let refname = validate_treeish(refname)?;
        let repo_s = repo_str(repo)?;
        let limit = limit.clamp(1, 100);
        let skip_s = skip.to_string();
        let limit_s = limit.to_string();

        // Empty / unborn → empty page.
        let rev = Command::new("git")
            .args([
                "-C",
                repo_s,
                "rev-parse",
                "--verify",
                &format!("{refname}^{{commit}}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
        if !rev.status.success() {
            return Ok(Vec::new());
        }

        let stdout = run_git_stdout(&[
            "-C",
            repo_s,
            "log",
            &format!("--skip={skip_s}"),
            &format!("--max-count={limit_s}"),
            "--format=%H%x00%h%x00%s%x00%an%x00%ae%x00%aI",
            refname,
        ])
        .await?;
        let text = String::from_utf8_lossy(&stdout);
        let mut out = Vec::new();
        for record in text.split('\n') {
            let record = record.trim_end_matches('\r');
            if record.is_empty() {
                continue;
            }
            if let Some(summary) = parse_commit_summary_record(record) {
                out.push(summary);
            }
        }
        Ok(out)
    }

    async fn show_commit(&self, repo: &Path, sha: &str) -> Result<CommitDetail, GitError> {
        let sha = validate_treeish(sha)?;
        let repo_s = repo_str(repo)?;

        let meta = run_git_stdout(&[
            "-C",
            repo_s,
            "show",
            "-s",
            "--format=%H%x00%h%x00%s%x00%b%x00%an%x00%ae%x00%aI%x00%P",
            sha,
        ])
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unknown revision")
                || msg.contains("bad object")
                || msg.contains("invalid object")
                || msg.contains("Not a valid object")
            {
                GitError::NotFound(format!("commit {sha}"))
            } else {
                e
            }
        })?;
        let meta_text = String::from_utf8_lossy(&meta);
        let meta_line = meta_text.trim();
        let parts: Vec<&str> = meta_line.split('\0').collect();
        if parts.len() < 8 {
            return Err(GitError::Process(format!(
                "unexpected git show format for {sha}"
            )));
        }
        let full_sha = parts[0].trim().to_string();
        let short_sha = parts[1].trim().to_string();
        let subject = parts[2].to_string();
        let body = parts[3].trim_end().to_string();
        let author_name = parts[4].to_string();
        let author_email = parts[5].to_string();
        let authored_at = parts[6].trim().to_string();
        let parents: Vec<String> = parts[7]
            .split_whitespace()
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect();

        let patch_bytes = run_git_stdout(&[
            "-C",
            repo_s,
            "show",
            "--format=",
            "--patch",
            "--find-renames",
            &full_sha,
        ])
        .await?;
        let patch_text = String::from_utf8_lossy(&patch_bytes);
        let (files, truncated) = parse_unified_diff_files(&patch_text, DIFF_SOFT_MAX_BYTES);

        Ok(CommitDetail {
            sha: full_sha,
            short_sha,
            subject,
            body,
            author_name,
            author_email,
            authored_at,
            parents,
            files,
            truncated,
        })
    }

    async fn diff(
        &self,
        repo: &Path,
        base: &str,
        head: &str,
    ) -> Result<DiffResult, GitError> {
        let base = validate_treeish(base)?;
        let head = validate_treeish(head)?;
        let repo_s = repo_str(repo)?;

        // Resolve both ends; identical trees → empty (not 500).
        for tip in [base, head] {
            let rev = Command::new("git")
                .args(["-C", repo_s, "rev-parse", "--verify", &format!("{tip}^{{commit}}")])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
            if !rev.status.success() {
                return Err(GitError::NotFound(format!("ref {tip}")));
            }
        }

        let range = format!("{base}...{head}");
        let output = Command::new("git")
            .args([
                "-C",
                repo_s,
                "diff",
                "--find-renames",
                "--patch",
                &range,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Empty / nothing to compare still often exits 0 with empty stdout.
            if output.stdout.is_empty()
                && (stderr.contains("unknown revision")
                    || stderr.contains("bad revision")
                    || stderr.contains("ambiguous"))
            {
                return Err(GitError::NotFound(format!("diff {range}")));
            }
            if output.stdout.is_empty() {
                return Ok(DiffResult {
                    base: base.to_string(),
                    head: head.to_string(),
                    files: Vec::new(),
                    empty: true,
                    truncated: false,
                });
            }
            return Err(GitError::Process(format!(
                "git diff failed: {}",
                stderr.trim()
            )));
        }

        if output.stdout.is_empty() {
            return Ok(DiffResult {
                base: base.to_string(),
                head: head.to_string(),
                files: Vec::new(),
                empty: true,
                truncated: false,
            });
        }

        let patch_text = String::from_utf8_lossy(&output.stdout);
        let (files, truncated) = parse_unified_diff_files(&patch_text, DIFF_SOFT_MAX_BYTES);
        let empty = files.is_empty();
        Ok(DiffResult {
            base: base.to_string(),
            head: head.to_string(),
            files,
            empty,
            truncated,
        })
    }

    async fn blame(
        &self,
        repo: &Path,
        refname: &str,
        path: &str,
    ) -> Result<BlameFile, GitError> {
        let refname = validate_treeish(refname)?;
        let path = validate_repo_rel_path(path)?;
        if path.is_empty() {
            return Err(GitError::InvalidArg("blame path required".into()));
        }
        let repo_s = repo_str(repo)?;

        let output = Command::new("git")
            .args([
                "-C",
                repo_s,
                "blame",
                "--line-porcelain",
                refname,
                "--",
                &path,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no such path")
                || stderr.contains("cannot find")
                || stderr.contains("Not a valid object")
                || stderr.contains("no such ref")
                || stderr.contains("fatal:")
            {
                return Err(GitError::NotFound(format!("blame {refname}:{path}")));
            }
            return Err(GitError::Process(format!(
                "git blame failed: {}",
                stderr.trim()
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let (lines, truncated) = parse_blame_porcelain(&text, BLAME_SOFT_MAX_LINES);
        Ok(BlameFile {
            path,
            ref_name: refname.to_string(),
            lines,
            truncated,
        })
    }

    async fn branch_create(
        &self,
        repo: &Path,
        name: &str,
        start: &str,
    ) -> Result<(), GitError> {
        let name = validate_treeish(name)?;
        let start = validate_treeish(start)?;
        let repo_s = repo_str(repo)?;
        // End-of-options so option-like names cannot become switches (CR-02).
        run_git(&["-C", repo_s, "branch", "--", name, start]).await?;
        Ok(())
    }

    async fn branch_rename(&self, repo: &Path, from: &str, to: &str) -> Result<(), GitError> {
        let from = validate_treeish(from)?;
        let to = validate_treeish(to)?;
        let repo_s = repo_str(repo)?;
        // Known flag -m, then end-of-options before user operands (CR-02).
        run_git(&["-C", repo_s, "branch", "-m", "--", from, to]).await?;
        Ok(())
    }

    async fn branch_delete(&self, repo: &Path, name: &str) -> Result<(), GitError> {
        let name = validate_treeish(name)?;
        let repo_s = repo_str(repo)?;
        // Force delete: forge UI confirms; bare repos have no "unmerged" worktree concept.
        // Known flag -D, then end-of-options before user operand (CR-02).
        run_git(&["-C", repo_s, "branch", "-D", "--", name]).await?;
        Ok(())
    }

    async fn archive(
        &self,
        repo: &Path,
        treeish: &str,
        format: ArchiveFormat,
        prefix: &str,
    ) -> Result<Vec<u8>, GitError> {
        let treeish = validate_treeish(treeish)?;
        let repo_s = repo_str(repo)?;
        let prefix = prefix.trim().trim_matches('/');
        if prefix.is_empty() || prefix.contains('\0') || prefix.contains("..") {
            return Err(GitError::InvalidArg(format!("invalid archive prefix: {prefix}")));
        }
        if prefix.chars().any(|c| matches!(c, ';' | '|' | '&' | '`' | '$' | '\n' | '\r')) {
            return Err(GitError::InvalidArg(format!("invalid archive prefix: {prefix}")));
        }
        let prefix_arg = format!("--prefix={prefix}/");
        let format_arg = format!("--format={}", format.as_git_format());
        // Known format/prefix flags, then end-of-options before user revision (CR-01).
        let args = [
            "-C",
            repo_s,
            "archive",
            format_arg.as_str(),
            prefix_arg.as_str(),
            "--",
            treeish,
        ];

        let result = tokio::time::timeout(ARCHIVE_TIMEOUT, run_git_stdout(&args))
            .await
            .map_err(|_| GitError::Process("git archive timed out".into()))?;

        match result {
            Ok(bytes) => Ok(bytes),
            Err(GitError::Process(msg)) => {
                let lower = msg.to_lowercase();
                if lower.contains("not a valid object")
                    || lower.contains("bad revision")
                    || lower.contains("unknown revision")
                    || lower.contains("does not exist")
                    || lower.contains("ambiguous argument")
                {
                    Err(GitError::NotFound(format!(
                        "archive unavailable for ref {treeish}"
                    )))
                } else {
                    Err(GitError::Process(msg))
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn gc(&self, repo: &Path) -> Result<(), GitError> {
        let repo_s = repo_str(repo)?;
        run_git(&["-C", repo_s, "gc", "--auto"]).await?;
        Ok(())
    }

    async fn merge_commit(
        &self,
        repo: &Path,
        base_ref: &str,
        head_sha: &str,
        message: &str,
    ) -> Result<String, GitError> {
        merge_via_worktree(repo, base_ref, head_sha, message, MergeMode::MergeCommit).await
    }

    async fn squash_merge(
        &self,
        repo: &Path,
        base_ref: &str,
        head_sha: &str,
        message: &str,
    ) -> Result<String, GitError> {
        merge_via_worktree(repo, base_ref, head_sha, message, MergeMode::Squash).await
    }

    async fn rebase_merge(
        &self,
        repo: &Path,
        base_ref: &str,
        head_sha: &str,
    ) -> Result<String, GitError> {
        merge_via_worktree(repo, base_ref, head_sha, "rebase", MergeMode::Rebase).await
    }

    async fn fetch_ref_from(
        &self,
        dest: &Path,
        source: &Path,
        refname: &str,
    ) -> Result<String, GitError> {
        let dest_abs = absolute_path(dest)?;
        let source_abs = absolute_path(source)?;
        let dest_s = dest_abs.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 dest: {}", dest_abs.display()))
        })?;
        let source_s = source_abs.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 source: {}", source_abs.display()))
        })?;
        let refname = validate_treeish(refname)?;
        // Fetch into a temporary ref then resolve SHA.
        let tmp_ref = format!("refs/octanest/fetch-tmp/{}", uuid_like());
        let refspec = format!("+{refname}:{tmp_ref}");
        run_git(&["-C", dest_s, "fetch", source_s, &refspec]).await?;
        let sha = run_git_stdout(&["-C", dest_s, "rev-parse", &tmp_ref]).await?;
        let sha = String::from_utf8_lossy(&sha).trim().to_string();
        let _ = run_git(&["-C", dest_s, "update-ref", "-d", &tmp_ref]).await;
        Ok(sha)
    }

    async fn clone_bare(&self, source: &Path, dest: &Path) -> Result<(), GitError> {
        let source_abs = absolute_path(source)?;
        let dest_abs = absolute_path(dest)?;
        let source_s = source_abs.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 source: {}", source_abs.display()))
        })?;
        let dest_s = dest_abs.to_str().ok_or_else(|| {
            GitError::InvalidArg(format!("non-utf8 dest: {}", dest_abs.display()))
        })?;
        if dest_abs.exists() {
            return Err(GitError::InvalidArg(format!(
                "dest already exists: {}",
                dest_abs.display()
            )));
        }
        if let Some(parent) = dest_abs.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        run_git(&["clone", "--bare", source_s, dest_s]).await?;
        Ok(())
    }

    async fn grep(
        &self,
        repo: &Path,
        treeish: &str,
        pattern: &str,
        pathspec: Option<&str>,
        max_matches: u32,
    ) -> Result<GrepResult, GitError> {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return Ok(GrepResult {
                hits: Vec::new(),
                truncated: false,
            });
        }
        if pattern.contains('\0') {
            return Err(GitError::InvalidArg("grep pattern contains NUL".into()));
        }
        let treeish = validate_treeish(treeish)?;
        let repo_s = repo_str(repo)?;
        let max_matches = max_matches.clamp(1, 10_000);
        let path_owned = match pathspec {
            Some(p) if !p.trim().is_empty() => Some(validate_repo_rel_path(p)?),
            _ => None,
        };

        // Empty / unborn → empty hits.
        let rev = Command::new("git")
            .args([
                "-C",
                repo_s,
                "rev-parse",
                "--verify",
                &format!("{treeish}^{{commit}}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
        if !rev.status.success() {
            return Ok(GrepResult {
                hits: Vec::new(),
                truncated: false,
            });
        }

        // -n line numbers, -I skip binary (D-SRCH-08), -e pattern as arg (no shell).
        // Tree-ish must NOT follow `--` or git treats it as a pathspec (work-tree error on bare).
        let mut cmd = Command::new("git");
        cmd.args(["-C", repo_s, "grep", "-n", "-I", "-e", pattern, treeish]);
        if let Some(ref p) = path_owned {
            cmd.args(["--", p]);
        }
        let output = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git grep: {e}")))?;

        match output.status.code() {
            Some(0) | Some(1) => {}
            _ => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(GitError::Process(format!(
                    "git grep failed (status {:?}): {}",
                    output.status.code(),
                    stderr.trim()
                )));
            }
        }

        // git grep prefixes matches with `treeish:` when searching a revision.
        // Format: `<treeish>:<path>:<line>:<content>`
        let text = String::from_utf8_lossy(&output.stdout);
        let mut hits = Vec::new();
        let mut truncated = false;
        for raw in text.split('\n') {
            let raw = raw.trim_end_matches('\r');
            if raw.is_empty() {
                continue;
            }
            if let Some(hit) = parse_grep_line(raw, treeish) {
                if hits.len() as u32 >= max_matches {
                    truncated = true;
                    break;
                }
                hits.push(hit);
            }
        }
        Ok(GrepResult { hits, truncated })
    }

    async fn log_search(
        &self,
        repo: &Path,
        refname: &str,
        grep: Option<&str>,
        author: Option<&str>,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<CommitSummary>, GitError> {
        let refname = validate_treeish(refname)?;
        let repo_s = repo_str(repo)?;
        let limit = limit.clamp(1, 100);
        let skip_s = skip.to_string();
        let limit_s = limit.to_string();

        let grep = grep.map(str::trim).filter(|s| !s.is_empty());
        let author = author.map(str::trim).filter(|s| !s.is_empty());
        if grep.is_none() && author.is_none() {
            return Ok(Vec::new());
        }
        if grep.is_some_and(|s| s.contains('\0')) || author.is_some_and(|s| s.contains('\0')) {
            return Err(GitError::InvalidArg("log_search filter contains NUL".into()));
        }

        let rev = Command::new("git")
            .args([
                "-C",
                repo_s,
                "rev-parse",
                "--verify",
                &format!("{refname}^{{commit}}"),
            ])
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
            repo_s.to_string(),
            "log".into(),
            format!("--skip={skip_s}"),
            format!("--max-count={limit_s}"),
            "--regexp-ignore-case".into(),
            "--format=%H%x00%h%x00%s%x00%an%x00%ae%x00%aI".into(),
        ];
        if let Some(g) = grep {
            args.push(format!("--grep={g}"));
        }
        if let Some(a) = author {
            args.push(format!("--author={a}"));
        }
        args.push(refname.to_string());
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

        let stdout = run_git_stdout(&arg_refs).await?;
        let text = String::from_utf8_lossy(&stdout);
        let mut out = Vec::new();
        for record in text.split('\n') {
            let record = record.trim_end_matches('\r');
            if record.is_empty() {
                continue;
            }
            if let Some(summary) = parse_commit_summary_record(record) {
                out.push(summary);
            }
        }
        Ok(out)
    }

    async fn log_path(
        &self,
        repo: &Path,
        refname: &str,
        path: &str,
        limit: u32,
    ) -> Result<Vec<CommitSummary>, GitError> {
        let refname = validate_treeish(refname)?;
        let repo_s = repo_str(repo)?;
        let path = path.trim().trim_start_matches('/');
        if path.is_empty() || path.contains('\0') {
            return Err(GitError::InvalidArg("log_path requires a non-empty path".into()));
        }
        if path
            .split('/')
            .any(|seg| seg.is_empty() || seg == "." || seg == "..")
        {
            return Err(GitError::InvalidArg("log_path path has invalid segments".into()));
        }
        let limit = limit.clamp(1, 100);
        let limit_s = limit.to_string();

        let rev = Command::new("git")
            .args([
                "-C",
                repo_s,
                "rev-parse",
                "--verify",
                &format!("{refname}^{{commit}}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
        if !rev.status.success() {
            return Ok(Vec::new());
        }

        let stdout = match run_git_stdout(&[
            "-C",
            repo_s,
            "log",
            &format!("--max-count={limit_s}"),
            "--format=%H%x00%h%x00%s%x00%an%x00%ae%x00%aI",
            refname,
            "--",
            path,
        ])
        .await
        {
            Ok(b) => b,
            Err(_) => return Ok(Vec::new()),
        };
        let text = String::from_utf8_lossy(&stdout);
        let mut out = Vec::new();
        for record in text.split('\n') {
            let record = record.trim_end_matches('\r');
            if record.is_empty() {
                continue;
            }
            if let Some(summary) = parse_commit_summary_record(record) {
                out.push(summary);
            }
        }
        Ok(out)
    }

    async fn path_last_commits(
        &self,
        repo: &Path,
        refname: &str,
        dir_path: &str,
        entry_names: &[String],
    ) -> Result<std::collections::HashMap<String, CommitSummary>, GitError> {
        let refname = validate_treeish(refname)?;
        let dir = dir_path.trim().trim_start_matches('/').trim_end_matches('/');
        if dir.contains('\0')
            || dir.split('/').any(|seg| seg == "." || seg == "..")
        {
            return Err(GitError::InvalidArg(
                "path_last_commits dir_path has invalid segments".into(),
            ));
        }

        let mut out = std::collections::HashMap::new();
        if entry_names.is_empty() {
            return Ok(out);
        }

        // Concurrency-capped parallel `log -1 -- path` (max 8).
        let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(8));
        let mut joins = Vec::with_capacity(entry_names.len());
        for name in entry_names {
            let name = name.trim().to_string();
            if name.is_empty() || name.contains('/') || name.contains('\0') || name == ".." {
                continue;
            }
            let full = if dir.is_empty() {
                name.clone()
            } else {
                format!("{dir}/{name}")
            };
            let repo = repo.to_path_buf();
            let refname = refname.to_string();
            let sem = sem.clone();
            let backend = self.clone();
            joins.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.ok()?;
                let commits = backend
                    .log_path(&repo, &refname, &full, 1)
                    .await
                    .ok()?;
                commits.into_iter().next().map(|c| (name, c))
            }));
        }
        for j in joins {
            if let Ok(Some((name, commit))) = j.await {
                out.insert(name, commit);
            }
        }
        Ok(out)
    }

    async fn rev_list_count(&self, repo: &Path, refname: &str) -> Result<u64, GitError> {
        let refname = validate_treeish(refname)?;
        let repo_s = repo_str(repo)?;
        let rev = Command::new("git")
            .args([
                "-C",
                repo_s,
                "rev-parse",
                "--verify",
                &format!("{refname}^{{commit}}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
        if !rev.status.success() {
            return Ok(0);
        }
        let stdout = run_git_stdout(&["-C", repo_s, "rev-list", "--count", refname]).await?;
        let text = String::from_utf8_lossy(&stdout);
        let n = text.trim().parse::<u64>().unwrap_or(0);
        Ok(n)
    }

    async fn shortlog(
        &self,
        repo: &Path,
        refname: &str,
        limit: u32,
    ) -> Result<Vec<ContributorSummary>, GitError> {
        let refname = validate_treeish(refname)?;
        let repo_s = repo_str(repo)?;
        let limit = limit.clamp(1, 100);

        let rev = Command::new("git")
            .args([
                "-C",
                repo_s,
                "rev-parse",
                "--verify",
                &format!("{refname}^{{commit}}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
        if !rev.status.success() {
            return Ok(Vec::new());
        }

        let stdout = run_git_stdout(&["-C", repo_s, "shortlog", "-sn", "-e", refname]).await?;
        let text = String::from_utf8_lossy(&stdout);
        let mut out = Vec::new();
        for line in text.split('\n') {
            if out.len() as u32 >= limit {
                break;
            }
            let line = line.trim_end_matches('\r').trim();
            if line.is_empty() {
                continue;
            }
            // "    42\tName <email@x>" or spaces then count then name <email>
            let rest = line.trim_start();
            let (count_s, after) = rest
                .split_once(|c: char| c.is_whitespace())
                .unwrap_or((rest, ""));
            let count: i64 = count_s.trim().parse().unwrap_or(0);
            if count <= 0 {
                continue;
            }
            let after = after.trim();
            let (name, email) = if let Some((n, e)) = after.rsplit_once(" <") {
                let email = e.trim().trim_end_matches('>').trim().to_string();
                (n.trim().to_string(), email)
            } else {
                (after.to_string(), String::new())
            };
            if name.is_empty() {
                continue;
            }
            out.push(ContributorSummary {
                name,
                email,
                commit_count: count,
            });
        }
        Ok(out)
    }

    async fn ls_tree_sized_blobs(
        &self,
        repo: &Path,
        treeish: &str,
        max_entries: u32,
    ) -> Result<Vec<SizedBlobEntry>, GitError> {
        let treeish = validate_treeish(treeish)?;
        let repo_s = repo_str(repo)?;
        let max_entries = max_entries.clamp(1, 200_000);

        let rev = Command::new("git")
            .args([
                "-C",
                repo_s,
                "rev-parse",
                "--verify",
                &format!("{treeish}^{{commit}}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| GitError::Process(format!("failed to spawn git: {e}")))?;
        if !rev.status.success() {
            return Ok(Vec::new());
        }

        // `-l` adds blob size after the OID; `-r` walks the full tree.
        let stdout = match run_git_stdout(&[
            "-C",
            repo_s,
            "ls-tree",
            "-r",
            "-l",
            "--full-tree",
            treeish,
        ])
        .await
        {
            Ok(b) => b,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("Not a valid object name")
                    || msg.contains("does not exist")
                    || msg.contains("not exist")
                {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        };
        let text = String::from_utf8_lossy(&stdout);
        let mut out = Vec::new();
        for line in text.lines() {
            if out.len() as u32 >= max_entries {
                break;
            }
            if let Some(entry) = parse_ls_tree_sized_blob_line(line) {
                out.push(entry);
            }
        }
        Ok(out)
    }
}

/// Parse `git ls-tree -r -l` lines: `mode type oid size\tpath` (blobs only).
fn parse_ls_tree_sized_blob_line(line: &str) -> Option<SizedBlobEntry> {
    let (meta, path) = line.split_once('\t')?;
    if path.is_empty() || path.contains('\0') {
        return None;
    }
    let mut parts = meta.split_whitespace();
    let _mode = parts.next()?;
    let kind = parts.next()?;
    if kind != "blob" {
        return None;
    }
    let _oid = parts.next()?;
    let size_s = parts.next()?;
    // Submodules / missing size show as `-`.
    if size_s == "-" {
        return None;
    }
    let size: u64 = size_s.parse().ok()?;
    Some(SizedBlobEntry {
        path: path.to_string(),
        size,
    })
}

fn parse_grep_line(line: &str, treeish: &str) -> Option<GrepHit> {
    let rest = line
        .strip_prefix(&format!("{treeish}:"))
        .unwrap_or(line);
    let (path, after_path) = rest.split_once(':')?;
    let (line_s, content) = after_path.split_once(':')?;
    let line_no: u32 = line_s.parse().ok()?;
    if path.is_empty() {
        return None;
    }
    Some(GrepHit {
        path: path.to_string(),
        line: line_no,
        content: content.to_string(),
    })
}

#[derive(Clone, Copy)]
enum MergeMode {
    MergeCommit,
    Squash,
    Rebase,
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{n:x}")
}

async fn merge_via_worktree(
    bare: &Path,
    base_ref: &str,
    head_sha: &str,
    message: &str,
    mode: MergeMode,
) -> Result<String, GitError> {
    let base_ref = validate_treeish(base_ref)?;
    let head_sha = validate_treeish(head_sha)?;
    if message.contains('\0') {
        return Err(GitError::InvalidArg("commit message contains NUL".into()));
    }
    let bare_abs = absolute_path(bare)?;
    let bare_s = bare_abs.to_str().ok_or_else(|| {
        GitError::InvalidArg(format!("non-utf8 bare path: {}", bare_abs.display()))
    })?;

    let tmp = tempfile::tempdir().map_err(GitError::Io)?;
    let work = tmp.path();
    let work_s = work
        .to_str()
        .ok_or_else(|| GitError::InvalidArg("non-utf8 temp worktree".into()))?;

    run_git(&["clone", bare_s, work_s]).await?;
    run_git(&["-C", work_s, "config", "user.email", "noreply@octanest.local"]).await?;
    run_git(&["-C", work_s, "config", "user.name", "Octanest"]).await?;
    if run_git(&[
        "-C",
        work_s,
        "checkout",
        "-B",
        base_ref,
        &format!("origin/{base_ref}"),
    ])
    .await
    .is_err()
    {
        run_git(&["-C", work_s, "checkout", "-B", base_ref, base_ref]).await?;
    }

    // Ensure head object is present (same-repo SHA already is).
    let _ = run_git(&["-C", work_s, "fetch", "origin", head_sha]).await;

    match mode {
        MergeMode::MergeCommit => {
            if let Err(e) = run_git(&[
                "-C",
                work_s,
                "merge",
                "--no-ff",
                "-m",
                message,
                head_sha,
            ])
            .await
            {
                let msg = e.to_string();
                if msg.to_lowercase().contains("conflict") {
                    return Err(GitError::Process(format!("merge conflict: {msg}")));
                }
                return Err(e);
            }
        }
        MergeMode::Squash => {
            if let Err(e) = run_git(&["-C", work_s, "merge", "--squash", head_sha]).await {
                let msg = e.to_string();
                if msg.to_lowercase().contains("conflict") {
                    return Err(GitError::Process(format!("merge conflict: {msg}")));
                }
                return Err(e);
            }
            run_git(&["-C", work_s, "commit", "-m", message]).await?;
        }
        MergeMode::Rebase => {
            run_git(&["-C", work_s, "checkout", "-B", "octanest-rebase-head", head_sha])
                .await?;
            if let Err(e) = run_git(&["-C", work_s, "rebase", base_ref]).await {
                let _ = run_git(&["-C", work_s, "rebase", "--abort"]).await;
                let msg = e.to_string();
                if msg.to_lowercase().contains("conflict") {
                    return Err(GitError::Process(format!("merge conflict: {msg}")));
                }
                return Err(e);
            }
            run_git(&["-C", work_s, "checkout", base_ref]).await?;
            run_git(&[
                "-C",
                work_s,
                "merge",
                "--ff-only",
                "octanest-rebase-head",
            ])
            .await?;
        }
    }

    let sha_bytes = run_git_stdout(&["-C", work_s, "rev-parse", "HEAD"]).await?;
    let sha = String::from_utf8_lossy(&sha_bytes).trim().to_string();
    let refspec = format!("HEAD:refs/heads/{base_ref}");
    run_git(&["-C", work_s, "push", "origin", &refspec]).await?;
    Ok(sha)
}

/// Resolve `path` against the process cwd when relative (seed push remote safety).
fn absolute_path(path: &Path) -> Result<PathBuf, GitError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(GitError::Io)?;
    Ok(cwd.join(path))
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
    use crate::backend::{ArchiveFormat, GitBackend, GitError, TreeEntryKind};

    #[test]
    fn absolute_path_keeps_absolute_and_joins_relative() {
        let abs = PathBuf::from("/var/repos/x.git");
        assert_eq!(absolute_path(&abs).unwrap(), abs);
        let rel = PathBuf::from("var/repos/x.git");
        let joined = absolute_path(&rel).unwrap();
        assert!(joined.is_absolute());
        assert!(joined.ends_with("var/repos/x.git"));
    }

    #[tokio::test]
    async fn seed_commit_works_with_relative_bare_path() {
        let root = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(root.path()).unwrap();
        let bare = PathBuf::from("relative-bare.git");
        let git = CliGitBackend::new();
        let result = async {
            git.init_bare(&bare, "main").await?;
            git.seed_commit(
                &bare,
                "main",
                "seed",
                &[("README.md".into(), b"# hi\n".to_vec())],
            )
            .await
        }
        .await;
        let _ = std::env::set_current_dir(&prev);
        result.expect("relative bare seed");
        assert!(root.path().join("relative-bare.git").join("HEAD").exists());
    }

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
    async fn grep_finds_seeded_line_and_empty_on_miss() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("grep.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "c",
            &[("src/a.txt".into(), b"alpha\nUNIQUE_GREP_TOKEN\nbeta\n".to_vec())],
        )
        .await
        .unwrap();
        let hit = git
            .grep(&bare, "main", "UNIQUE_GREP_TOKEN", None, 50)
            .await
            .expect("grep");
        assert_eq!(hit.hits.len(), 1);
        assert_eq!(hit.hits[0].path, "src/a.txt");
        assert_eq!(hit.hits[0].line, 2);
        assert!(hit.hits[0].content.contains("UNIQUE_GREP_TOKEN"));
        assert!(!hit.truncated);

        let miss = git
            .grep(&bare, "main", "no_such_token_zzz", None, 50)
            .await
            .expect("grep miss");
        assert!(miss.hits.is_empty());
        assert!(!miss.truncated);
    }

    #[tokio::test]
    async fn grep_skips_binary_with_i_and_truncates() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("grep_cap.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        let mut text = String::new();
        for i in 0..20 {
            text.push_str(&format!("CAP_TOKEN line {i}\n"));
        }
        // Binary-ish file with NUL — git grep -I should skip.
        let mut bin = b"CAP_TOKEN\0binary".to_vec();
        bin.extend_from_slice(&[0u8; 8]);
        git.seed_commit(
            &bare,
            "main",
            "c",
            &[
                ("text.txt".into(), text.into_bytes()),
                ("bin.dat".into(), bin),
            ],
        )
        .await
        .unwrap();
        let capped = git
            .grep(&bare, "main", "CAP_TOKEN", None, 5)
            .await
            .expect("grep cap");
        assert_eq!(capped.hits.len(), 5);
        assert!(capped.truncated);
        assert!(
            capped.hits.iter().all(|h| h.path != "bin.dat"),
            "binary file should be skipped with -I"
        );
    }

    #[tokio::test]
    async fn log_search_matches_message_and_author() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("log_search.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "UNIQUE_COMMIT_MSG_TOKEN",
            &[("a.txt".into(), b"one\n".to_vec())],
        )
        .await
        .unwrap();
        let by_msg = git
            .log_search(
                &bare,
                "main",
                Some("UNIQUE_COMMIT_MSG_TOKEN"),
                None,
                0,
                10,
            )
            .await
            .expect("log_search msg");
        assert_eq!(by_msg.len(), 1);
        assert!(by_msg[0].subject.contains("UNIQUE_COMMIT_MSG_TOKEN"));

        let by_author = git
            .log_search(&bare, "main", None, Some("Octanest"), 0, 10)
            .await
            .expect("log_search author");
        assert!(!by_author.is_empty());

        let miss = git
            .log_search(&bare, "main", Some("zzz_no_msg"), None, 0, 10)
            .await
            .expect("log_search miss");
        assert!(miss.is_empty());
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

    #[tokio::test]
    async fn log_returns_paged_commit_summaries_for_ref() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("log.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "first commit",
            &[("a.txt".into(), b"one\n".to_vec())],
        )
        .await
        .unwrap();
        // Second commit via clone + push.
        let wt = tempfile::tempdir().unwrap();
        let wt_s = wt.path().to_str().unwrap();
        let bare_s = bare.to_str().unwrap();
        run_git(&["clone", bare_s, wt_s]).await.unwrap();
        tokio::fs::write(wt.path().join("a.txt"), b"two\n")
            .await
            .unwrap();
        run_git(&["-C", wt_s, "add", "a.txt"]).await.unwrap();
        run_git(&["-C", wt_s, "commit", "-m", "second commit"])
            .await
            .unwrap();
        run_git(&["-C", wt_s, "push", "origin", "HEAD:main"])
            .await
            .unwrap();

        let page = git.log(&bare, "main", 0, 10).await.expect("log");
        assert!(
            page.len() >= 2,
            "expected at least 2 commits, got {}",
            page.len()
        );
        assert_eq!(page[0].subject, "second commit");
        assert!(!page[0].sha.is_empty());
        assert!(!page[0].short_sha.is_empty());
        assert!(!page[0].author_name.is_empty());
        assert!(!page[0].authored_at.is_empty());

        let skipped = git.log(&bare, "main", 1, 1).await.expect("log skip");
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0].subject, "first commit");
    }

    #[tokio::test]
    async fn show_commit_returns_files_and_unified_patch() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("show.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "seed subject",
            &[("hello.txt".into(), b"hello\nworld\n".to_vec())],
        )
        .await
        .unwrap();
        let bare_s = bare.to_str().unwrap();
        let sha_bytes = run_git_stdout(&["-C", bare_s, "rev-parse", "main"])
            .await
            .unwrap();
        let sha = String::from_utf8_lossy(&sha_bytes).trim().to_string();

        let detail = git.show_commit(&bare, &sha).await.expect("show_commit");
        assert_eq!(detail.sha, sha);
        assert_eq!(detail.subject, "seed subject");
        assert!(
            !detail.files.is_empty(),
            "expected at least one file in commit"
        );
        let hello = detail
            .files
            .iter()
            .find(|f| f.path == "hello.txt")
            .expect("hello.txt");
        assert!(
            hello.patch.contains("hello") || hello.patch.contains("+++"),
            "expected unified patch content, got: {}",
            hello.patch
        );
    }

    #[tokio::test]
    async fn diff_identical_refs_returns_empty_not_error() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("diff.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "only",
            &[("x.txt".into(), b"x\n".to_vec())],
        )
        .await
        .unwrap();

        let result = git
            .diff(&bare, "main", "main")
            .await
            .expect("diff identical must not 500");
        assert!(result.empty, "identical refs should be empty");
        assert!(result.files.is_empty());
    }

    #[tokio::test]
    async fn blame_returns_per_line_meta_for_text_file() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("blame.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "blame me",
            &[("lines.txt".into(), b"alpha\nbeta\n".to_vec())],
        )
        .await
        .unwrap();

        let blame = git
            .blame(&bare, "main", "lines.txt")
            .await
            .expect("blame");
        assert_eq!(blame.path, "lines.txt");
        assert!(
            blame.lines.len() >= 2,
            "expected >=2 blame lines, got {}",
            blame.lines.len()
        );
        assert!(!blame.lines[0].sha.is_empty());
        assert!(!blame.lines[0].author_name.is_empty());
        assert_eq!(blame.lines[0].line_number, 1);
        assert!(blame.lines[0].content.contains("alpha"));
    }

    /// Filter: `git_archive_formats` — zip + tar.gz via CliGitBackend (GIT-07).
    #[tokio::test]
    async fn git_archive_formats_zip_and_tar_gz() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("arch.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "archive seed",
            &[("hello.txt".into(), b"hello archive\n".to_vec())],
        )
        .await
        .unwrap();

        let zip = git
            .archive(&bare, "main", ArchiveFormat::Zip, "arch")
            .await
            .expect("zip archive");
        assert!(
            !zip.is_empty(),
            "zip archive bytes must be non-empty"
        );
        assert_eq!(&zip[0..2], b"PK", "zip should start with PK magic");

        let tar_gz = git
            .archive(&bare, "main", ArchiveFormat::TarGz, "arch")
            .await
            .expect("tar.gz archive");
        assert!(
            !tar_gz.is_empty(),
            "tar.gz archive bytes must be non-empty"
        );
        // gzip magic 1f 8b
        assert_eq!(&tar_gz[0..2], &[0x1f, 0x8b], "tar.gz should be gzip");
    }

    #[tokio::test]
    async fn git_archive_empty_repo_returns_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("empty-arch.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();

        let err = git
            .archive(&bare, "main", ArchiveFormat::Zip, "empty-arch")
            .await
            .expect_err("empty repo archive must fail");
        match err {
            GitError::NotFound(_) => {}
            other => panic!("expected NotFound for empty archive, got {other}"),
        }
    }

    #[tokio::test]
    async fn gc_runs_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("gc.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "gc seed",
            &[("a.txt".into(), b"a\n".to_vec())],
        )
        .await
        .unwrap();
        git.gc(&bare).await.expect("gc");
    }

    async fn push_branch_with_file(
        bare: &Path,
        branch: &str,
        from: &str,
        file: &str,
        content: &[u8],
        message: &str,
    ) {
        let wt = tempfile::tempdir().unwrap();
        let wt_s = wt.path().to_str().unwrap();
        let bare_s = bare.to_str().unwrap();
        run_git(&["clone", bare_s, wt_s]).await.unwrap();
        run_git(&["-C", wt_s, "checkout", "-B", branch, from])
            .await
            .unwrap();
        tokio::fs::write(wt.path().join(file), content)
            .await
            .unwrap();
        run_git(&["-C", wt_s, "add", file]).await.unwrap();
        run_git(&["-C", wt_s, "commit", "-m", message])
            .await
            .unwrap();
        let refspec = format!("HEAD:refs/heads/{branch}");
        run_git(&["-C", wt_s, "push", "origin", &refspec])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn merge_commit_creates_merge_on_base() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("merge.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "seed",
            &[("a.txt".into(), b"base\n".to_vec())],
        )
        .await
        .unwrap();
        push_branch_with_file(&bare, "feature", "main", "b.txt", b"feat\n", "feat")
            .await;
        let head = String::from_utf8_lossy(
            &run_git_stdout(&[
                "-C",
                bare.to_str().unwrap(),
                "rev-parse",
                "refs/heads/feature",
            ])
            .await
            .unwrap(),
        )
        .trim()
        .to_string();
        let sha = git
            .merge_commit(&bare, "main", &head, "Merge feature")
            .await
            .expect("merge_commit");
        assert_eq!(sha.len(), 40);
        let tip = String::from_utf8_lossy(
            &run_git_stdout(&["-C", bare.to_str().unwrap(), "rev-parse", "refs/heads/main"])
                .await
                .unwrap(),
        )
        .trim()
        .to_string();
        assert_eq!(tip, sha);
    }

    #[tokio::test]
    async fn squash_merge_single_commit_on_base() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("squash.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "seed",
            &[("a.txt".into(), b"base\n".to_vec())],
        )
        .await
        .unwrap();
        push_branch_with_file(&bare, "feature", "main", "c.txt", b"sq\n", "sq")
            .await;
        let head = String::from_utf8_lossy(
            &run_git_stdout(&[
                "-C",
                bare.to_str().unwrap(),
                "rev-parse",
                "refs/heads/feature",
            ])
            .await
            .unwrap(),
        )
        .trim()
        .to_string();
        let sha = git
            .squash_merge(&bare, "main", &head, "Squash feature")
            .await
            .expect("squash_merge");
        assert_eq!(sha.len(), 40);
    }

    #[tokio::test]
    async fn rebase_merge_fast_forwards_base() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("rebase.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "seed",
            &[("a.txt".into(), b"base\n".to_vec())],
        )
        .await
        .unwrap();
        push_branch_with_file(&bare, "feature", "main", "d.txt", b"rb\n", "rb")
            .await;
        let head = String::from_utf8_lossy(
            &run_git_stdout(&[
                "-C",
                bare.to_str().unwrap(),
                "rev-parse",
                "refs/heads/feature",
            ])
            .await
            .unwrap(),
        )
        .trim()
        .to_string();
        let sha = git
            .rebase_merge(&bare, "main", &head)
            .await
            .expect("rebase_merge");
        assert_eq!(sha.len(), 40);
    }

    #[tokio::test]
    async fn merge_commit_conflict_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = tmp.path().join("conflict.git");
        let git = CliGitBackend::new();
        git.init_bare(&bare, "main").await.unwrap();
        git.seed_commit(
            &bare,
            "main",
            "seed",
            &[("clash.txt".into(), b"base\n".to_vec())],
        )
        .await
        .unwrap();
        // Divergent edits on same file.
        push_branch_with_file(
            &bare,
            "feature",
            "main",
            "clash.txt",
            b"feature\n",
            "feat clash",
        )
        .await;
        // Advance main with conflicting content.
        push_branch_with_file(&bare, "main", "main", "clash.txt", b"mainline\n", "main clash")
            .await;
        let head = String::from_utf8_lossy(
            &run_git_stdout(&[
                "-C",
                bare.to_str().unwrap(),
                "rev-parse",
                "refs/heads/feature",
            ])
            .await
            .unwrap(),
        )
        .trim()
        .to_string();
        let err = git
            .merge_commit(&bare, "main", &head, "Merge conflict")
            .await
            .expect_err("expected conflict");
        let msg = err.to_string().to_lowercase();
        assert!(
            msg.contains("conflict") || msg.contains("failed"),
            "unexpected err: {msg}"
        );
    }

    /// D-FORK-02/03: clone_bare must install hooks/update (same as init_bare).
    #[tokio::test]
    async fn clone_bare_installs_protection_hooks() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("src.git");
        let dest = tmp.path().join("dest.git");
        let git = CliGitBackend::new();
        git.init_bare(&source, "main").await.unwrap();
        git.seed_commit(
            &source,
            "main",
            "seed",
            &[("README.md".into(), b"hi\n".to_vec())],
        )
        .await
        .unwrap();
        // Strip hooks so clone cannot inherit a valid update file from source.
        let _ = tokio::fs::remove_file(source.join("hooks").join("update")).await;

        git.clone_bare(&source, &dest).await.expect("clone_bare");
        let update = dest.join("hooks").join("update");
        let meta = tokio::fs::metadata(&update)
            .await
            .expect("hooks/update must exist after clone_bare");
        assert!(meta.is_file(), "hooks/update must be a file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                meta.permissions().mode() & 0o111,
                0o111,
                "hooks/update must be executable"
            );
        }
        let body = tokio::fs::read_to_string(&update).await.unwrap();
        assert!(
            body.contains("OCTANEST_ENV") && body.contains("production|cloud"),
            "cloned hook script must include D-PKG-02 gate"
        );
    }

    /// D-FORK-04: hook install failure after clone propagates as Err.
    #[tokio::test]
    async fn clone_bare_fails_when_hooks_update_is_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("src.git");
        let dest = tmp.path().join("dest.git");
        let git = CliGitBackend::new();
        git.init_bare(&source, "main").await.unwrap();
        git.seed_commit(
            &source,
            "main",
            "seed",
            &[("README.md".into(), b"hi\n".to_vec())],
        )
        .await
        .unwrap();
        // Make source hooks/update a directory so bare clone copies it and
        // install_protection_hooks cannot overwrite with a file.
        let src_update = source.join("hooks").join("update");
        let _ = tokio::fs::remove_file(&src_update).await;
        tokio::fs::create_dir_all(&src_update).await.unwrap();

        let err = git
            .clone_bare(&source, &dest)
            .await
            .expect_err("hook install must fail");
        assert!(
            matches!(err, GitError::Io(_) | GitError::Process(_)),
            "unexpected error variant: {err:?}"
        );
    }
}
