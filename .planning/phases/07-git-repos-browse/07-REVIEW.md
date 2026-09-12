---
phase: 07-git-repos-browse
reviewed: 2026-09-12T19:15:00Z
depth: standard
files_reviewed: 42
files_reviewed_list:
  - crates/octanest-git/src/cli.rs
  - crates/octanest-git/src/backend.rs
  - crates/octanest-git/src/lib.rs
  - crates/octanest-git/src/version.rs
  - crates/octanest-api/src/repo/acl.rs
  - crates/octanest-api/src/repo/mod.rs
  - crates/octanest-api/src/repo/templates.rs
  - crates/octanest-api/src/routes/repo_raw.rs
  - crates/octanest-api/src/git/mod.rs
  - crates/octanest-api/src/rpc.rs
  - crates/octanest-api/src/app.rs
  - crates/octanest-api/src/main.rs
  - crates/octanest-api/src/auth/admin.rs
  - crates/octanest-api/src/jobs/mod.rs
  - crates/octanest-api/src/jobs/reconcile.rs
  - crates/octanest-api/src/jobs/schedule.rs
  - crates/octanest-db/src/repositories.rs
  - crates/octanest-db/migrations/sqlite/0007_repositories.sql
  - crates/octanest-db/migrations/postgres/0007_repositories.sql
  - crates/octanest-db/migrations/mysql/0007_repositories.sql
  - crates/octanest-core/src/repo_types.rs
  - apps/web/src/lib/markdown.ts
  - apps/web/src/lib/markdown.test.ts
  - apps/web/src/lib/highlight.ts
  - apps/web/src/lib/repo-browse.ts
  - apps/web/src/components/repo/blob-viewer.tsrx
  - apps/web/src/components/repo/readme-panel.tsrx
  - apps/web/src/components/repo/repo-chrome.tsrx
  - apps/web/src/components/repo/clone-box.tsrx
  - apps/web/src/components/repo/file-tree.tsrx
  - apps/web/src/components/repo/repo-not-found.tsrx
  - apps/web/src/routes/$owner.$repo.settings.tsrx
  - apps/web/src/routes/new.tsrx
  - apps/web/src/routes/$owner.$repo.tsrx
  - apps/web/src/routes/$owner.$repo.tree.$.tsrx
  - apps/web/src/routes/$owner.$repo.blob.$.tsrx
  - crates/octanest-api/tests/repo_private_404.rs
  - crates/octanest-api/tests/repo_branch_soft_protect.rs
  - crates/octanest-api/tests/repo_settings_visibility_delete.rs
  - crates/octanest-api/tests/repo_archive.rs
  - packages/api-client/src/index.ts
  - Cargo.toml
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-09-12T19:15:00Z
**Depth:** standard
**Files Reviewed:** 42
**Status:** issues_found

## Summary

Phase 07 ACL (`resolve_repo_for_read` / owner mutate → identical `repo.not_found`), soft-delete SQL filters, markdown `rehype-sanitize`, and RPC auth gating for mutate/admin paths look sound. Two **critical** git argv issues remain: `validate_treeish` / archive treeish validation does not reject leading `-`, enabling option injection — including **arbitrary file create/truncate via `git archive --output=`** on public repos (anonymous), and **default-branch force-delete via `repo.branchCreate` with `branch="-D"`**.

## Critical Issues

### CR-01: Archive treeish option injection writes/truncates arbitrary files

**File:** `crates/octanest-git/src/cli.rs:885-903` (caller: `crates/octanest-api/src/routes/repo_raw.rs:54-76`, `239-272`)
**Issue:** `archive` passes user `treeish` as a trailing argv after `--format` / `--prefix`. Values starting with `-` are accepted by `validate_treeish` / `validate_archive_treeish`. Git treats `--output=/path` as an option and **creates/truncates that path** (verified: empty file created even when the command then fails for missing tree-ish). Any caller who can hit `GET /api/repos/{owner}/{repo}/archive/--output=<path>.zip` on a readable (e.g. public) repo can write as the API process user — disk wipe/truncate risk outside the repos root.
**Fix:** Reject option-like refs and force end-of-options before the revision:

```rust
fn validate_treeish(treeish: &str) -> Result<&str, GitError> {
    let t = treeish.trim();
    if t.is_empty() || t.starts_with('-') || t.contains('\0') || t.contains("..") {
        return Err(GitError::InvalidArg(format!("invalid treeish: {treeish}")));
    }
    // ... existing metacharacter checks ...
    Ok(t)
}

// In archive / all git calls that take a revision:
run_git_stdout(&["-C", repo_s, "archive", format_arg, prefix_arg, "--", treeish])
```

Apply the same `starts_with('-')` + `"--"` pattern in `repo_raw::validate_archive_treeish` / `validate_ref`.

### CR-02: `repo.branchCreate` with `branch="-D"` force-deletes the start ref (bypasses soft-protect)

**File:** `crates/octanest-git/src/cli.rs:849-859` (caller: `crates/octanest-api/src/repo/mod.rs:466-495`)
**Issue:** `branch_create` runs `git branch <name> <start>`. If `name` is `-D` (or `-d` / `-M` / …) and `start` is `main`, this becomes `git branch -D main`, **force-deleting the default branch** without going through `branch_delete` soft-protect (`repo.default_branch_protected`). Confirmed with system git. Owner-only, but destroys protected refs and violates D-28.
**Fix:** Reject leading `-` in branch/ref validators (same as CR-01) and insert `"--"` before name/start:

```rust
run_git(&["-C", repo_s, "branch", "--", name, start]).await?;
// similarly for branch_rename / branch_delete
```

Also reject reserved option tokens (`-d`, `-D`, `-m`, `-M`, `-f`, …) explicitly in API input validation.

## Warnings

### WR-01: `repo.create` leaves DB row if `init_bare` / `seed_commit` fails

**File:** `crates/octanest-api/src/repo/mod.rs:662-709`
**Issue:** After a successful `insert_repository`, git failures return `repo.git_init_failed` / `repo.git_seed_failed` without rolling back or soft-deleting the row. The name remains taken (`deleted_at IS NULL`), so the owner cannot recreate; reconcile will not purge it (row is “known”).
**Fix:** On git failure after insert, soft-delete or hard-delete the row (and remove any partial bare dir under `repos_dir`), or wrap create in a compensating transaction pattern.

### WR-02: Raw HTTP refs reject `/` while archive/RPC allow slashy branch names

**File:** `crates/octanest-api/src/routes/repo_raw.rs:42-51` vs `54-76`
**Issue:** `validate_ref` for raw blobs rejects `/`, so branches like `feature/x` cannot be downloaded via `/raw/...`, while `validate_archive_treeish` and CLI `validate_treeish` allow `/`. Inconsistent browse vs archive/raw behavior.
**Fix:** Align raw ref validation with archive (allow `/`, still reject `..`, NUL, metacharacters, and leading `-`).

### WR-03: UI `parseRefAndPath` cannot represent refs containing `/`

**File:** `apps/web/src/lib/repo-browse.ts:14-25`
**Issue:** Only the first splat segment is treated as `ref`; `feature/foo/path` is parsed as ref=`feature`, path=`foo/path`. Breaks tree/blob/blame URLs for hierarchical branch names (D-17).
**Fix:** Resolve ref against `repo.refs` (longest prefix match) before splitting path, or use a delimiter that cannot appear in refs.

## Info

### IN-01: Markdown XSS path is correctly sanitized

**File:** `apps/web/src/lib/markdown.ts:12-20`
**Issue:** None — `rehype-sanitize` is last; tests assert script/onerror stripping. README `dangerouslySetInnerHTML` is acceptable given sanitize.
**Fix:** N/A (keep sanitize last; avoid `allowDangerousHtml: true` on `remarkRehype`).

### IN-02: Highlight HTML injects `data-language` without escaping

**File:** `apps/web/src/lib/highlight.ts:130-135`
**Issue:** `lang` is interpolated into HTML. Currently constrained to loaded language ids / `plaintext`, so not exploitable today.
**Fix:** Escape `lang` (or only allow `[a-z0-9-]+`) before interpolation.

---

_Reviewed: 2026-09-12T19:15:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
