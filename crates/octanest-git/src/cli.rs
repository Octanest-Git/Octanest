//! CLI-backed [`GitBackend`] via `tokio::process::Command` argv arrays (never `sh -c`).

use std::path::{Component, Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::backend::{
    ArchiveFormat, BlameFile, BlameLine, CommitDetail, CommitSummary, DiffFile, DiffResult,
    GitBackend, GitError, GitRef, TreeEntry, TreeEntryKind, ARCHIVE_TIMEOUT, BLAME_SOFT_MAX_LINES,
    DIFF_SOFT_MAX_BYTES,
};

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

/// Reject NUL / `..` / absolute-looking refs (T-07-15 / T-07-17).
fn validate_treeish(treeish: &str) -> Result<&str, GitError> {
    let t = treeish.trim();
    if t.is_empty() || t.contains('\0') || t.contains("..") {
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
        run_git(&["-C", repo_s, "branch", name, start]).await?;
        Ok(())
    }

    async fn branch_rename(&self, repo: &Path, from: &str, to: &str) -> Result<(), GitError> {
        let from = validate_treeish(from)?;
        let to = validate_treeish(to)?;
        let repo_s = repo_str(repo)?;
        run_git(&["-C", repo_s, "branch", "-m", from, to]).await?;
        Ok(())
    }

    async fn branch_delete(&self, repo: &Path, name: &str) -> Result<(), GitError> {
        let name = validate_treeish(name)?;
        let repo_s = repo_str(repo)?;
        // Force delete: forge UI confirms; bare repos have no "unmerged" worktree concept.
        run_git(&["-C", repo_s, "branch", "-D", name]).await?;
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
        let args = [
            "-C",
            repo_s,
            "archive",
            format_arg.as_str(),
            prefix_arg.as_str(),
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
}
