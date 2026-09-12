---
phase: 07-git-repos-browse
verified: 2026-09-12T19:25:00Z
status: gaps_found
score: 6/7 must-haves verified
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/phases/07-git-repos-browse/07-00-PLAN.md
  - .planning/phases/07-git-repos-browse/07-00-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-01-PLAN.md
  - .planning/phases/07-git-repos-browse/07-01-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-02-PLAN.md
  - .planning/phases/07-git-repos-browse/07-02-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-03-PLAN.md
  - .planning/phases/07-git-repos-browse/07-03-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-04-PLAN.md
  - .planning/phases/07-git-repos-browse/07-04-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-05-PLAN.md
  - .planning/phases/07-git-repos-browse/07-05-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-06-PLAN.md
  - .planning/phases/07-git-repos-browse/07-06-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-07-PLAN.md
  - .planning/phases/07-git-repos-browse/07-07-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-08-PLAN.md
  - .planning/phases/07-git-repos-browse/07-08-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-09-PLAN.md
  - .planning/phases/07-git-repos-browse/07-09-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-10-PLAN.md
  - .planning/phases/07-git-repos-browse/07-10-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-11-PLAN.md
  - .planning/phases/07-git-repos-browse/07-11-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-12-PLAN.md
  - .planning/phases/07-git-repos-browse/07-12-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-13-PLAN.md
  - .planning/phases/07-git-repos-browse/07-13-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-14-PLAN.md
  - .planning/phases/07-git-repos-browse/07-14-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-15-PLAN.md
  - .planning/phases/07-git-repos-browse/07-15-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-16-PLAN.md
  - .planning/phases/07-git-repos-browse/07-16-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-17-PLAN.md
  - .planning/phases/07-git-repos-browse/07-17-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-18-PLAN.md
  - .planning/phases/07-git-repos-browse/07-18-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-REVIEW.md
  - apps/web/src/components/repo/clone-box.tsrx
  - apps/web/src/lib/highlight.ts
  - apps/web/src/lib/markdown.ts
  - apps/web/src/lib/repo-browse.ts
  - apps/web/src/routes/$owner.$repo.blob.$.tsrx
  - apps/web/src/routes/$owner.$repo.branches.tsrx
  - apps/web/src/routes/$owner.$repo.index.tsrx
  - apps/web/src/routes/$owner.$repo.settings.tsrx
  - apps/web/src/routes/$owner.$repo.tags.tsrx
  - apps/web/src/routes/$owner.$repo.tree.$.tsrx
  - apps/web/src/routes/new.tsrx
  - crates/octanest-api/src/app.rs
  - crates/octanest-api/src/main.rs
  - crates/octanest-api/src/repo/acl.rs
  - crates/octanest-api/src/repo/mod.rs
  - crates/octanest-api/src/repo/templates.rs
  - crates/octanest-api/src/routes/repo_raw.rs
  - crates/octanest-api/src/rpc.rs
  - crates/octanest-core/src/repo_types.rs
  - crates/octanest-db/migrations/mysql/0007_repositories.sql
  - crates/octanest-db/migrations/postgres/0007_repositories.sql
  - crates/octanest-db/migrations/sqlite/0007_repositories.sql
  - crates/octanest-db/src/repositories.rs
  - crates/octanest-git/src/backend.rs
  - crates/octanest-git/src/cli.rs
  - crates/octanest-git/src/lib.rs
  - crates/octanest-git/src/version.rs
  - docs/ARCHITECTURE.md
  - docs/CONFIGURATION.md
covered_digest: "v1:sha256:59c3cc1d8ad96646151b0714c6a5deea0b258271c92ebb7c036bc5d16524bc44"
behavior_unverified: 3
overrides_applied: 0
decision_coverage:
  honored: 38
  total: 38
  not_honored: []
gaps:
  - truth: "User can create, rename, and delete branches from the web UI where permitted (soft-protect default branch — D-28 / GIT-06)"
    status: failed
    reason: "07-REVIEW CR-02 unfixed: validate_treeish does not reject leading '-', and branch_create runs `git branch <name> <start>` without `--`. Owner can call repo.branchCreate with branch=\"-D\" and start=default branch to force-delete the protected default branch, bypassing soft-protect on branchDelete/branchRename. Reproduced with system git; code at cli.rs:58-72 and cli.rs:849-859 still matches REVIEW."
    artifacts:
      - path: crates/octanest-git/src/cli.rs
        issue: "validate_treeish allows option-like refs; branch_create/rename/delete lack end-of-options `--`"
      - path: crates/octanest-api/src/repo/mod.rs
        issue: "branch_create does not reject reserved option tokens (-d/-D/-m/-M/-f)"
    missing:
      - "Reject treeish/branch names starting with `-` (and reserved git option tokens) in validate_treeish + API validators"
      - "Pass `--` before name/start in branch_create, before from/to in branch_rename, before name in branch_delete"
      - "Integration test: repo.branchCreate with branch=\"-D\" must fail and leave default branch intact"
  - truth: "Archive/treeish argv is safe — refs validated so git cannot interpret user input as CLI options (GIT-07 / ASSUME refs validated before argv)"
    status: failed
    reason: "07-REVIEW CR-01 unfixed: validate_treeish / validate_archive_treeish accept leading `-`; archive argv places treeish after --format/--prefix without `--`. Public-repo archive URL with treeish `--output=<path>` creates/truncates files as the API process user. Reproduced: empty file created via `git archive ... --output=<path>` option-like treeish."
    artifacts:
      - path: crates/octanest-git/src/cli.rs
        issue: "archive() does not reject leading `-` or insert `--` before treeish"
      - path: crates/octanest-api/src/routes/repo_raw.rs
        issue: "validate_archive_treeish still allows option-like treeish"
    missing:
      - "Reject starts_with('-') in validate_treeish and validate_archive_treeish / validate_ref"
      - "Use `git archive ... -- <treeish>` (and same pattern for other revision argv)"
      - "Regression test for archive/--output injection rejection"
behavior_unverified_items:
  - truth: "Syntax highlighting visual fidelity for .tsrx/.ripple held for human UAT (07-00 backstop)"
    test: "Open a seeded blob for .tsrx and .ripple in the Code UI"
    expected: "Tokens highlight via in-repo grammars (not plain TS/JS alias look)"
    why_human: "verification: backstop — presence of grammars/tests does not prove visual fidelity"
  - truth: "/new description Textarea wraps without horizontal overflow (07-03 backstop)"
    test: "Paste a long unbroken description on /new"
    expected: "Text wraps; no horizontal page overflow"
    why_human: "verification: backstop — layout cannot be proven by grep"
  - truth: "Long paths/descriptions ellipsis or wrap — held-out visual (07-15 backstop)"
    test: "Browse a deep/long file path in tree/blob chrome"
    expected: "Ellipsis or wrap per UI-SPEC; layout remains usable"
    why_human: "verification: backstop — visual layout only"
---

# Phase 7: Git Repos & Browse Verification Report

**Phase Goal:** Users can create filesystem-backed repos and browse history in the UI via system `git` CLI behind a `GitBackend` seam, with a documented future gitoxide path

**Verified:** 2026-09-12T19:25:00Z  
**Status:** gaps_found  
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Authenticated (and verified, on cloud) user can create a public or private repository | ✓ VERIFIED | `repo.create` in `rpc.rs`/`repo/mod.rs`; `/new` calls `apiClient.repo.create`; verify wall + `auth.email_unverified`; tests `repo_create_*` present |
| 2 | User can browse files, commits, branches, and tags in the web UI and download a source archive for a ref | ✓ VERIFIED | Routes tree/blob/commits/branches/tags wired; `clone-box` archive URLs; `repo_archive_zip_and_tar_gz_nonempty_for_seeded_ref` **PASS**. **Note:** CR-01 security hole is a separate failed truth (#7-adjacent gap) |
| 3 | User can create, rename, and delete branches from the web UI where permitted | ✗ FAILED | Happy-path API/UI + soft-protect on rename/delete exist (`repo_branch_soft_protect_blocks_default_rename_and_delete` **PASS**), but **CR-02** lets `branchCreate(branch="-D")` force-delete the default branch — soft-protect / D-28 / “where permitted” broken |
| 4 | Repository objects live on the local filesystem (volume-backed), and git ops use system `git` CLI with docs allowing future gitoxide swap | ✓ VERIFIED | `OCTANEST_REPOS_DIR` + Compose `./var/repos:/var/repos`; `CliGitBackend` on `AppState`; `assert_git_version` fail-boot; `docs/ARCHITECTURE.md` Cli now / Gix later; no `GixGitBackend` body |
| 5 | Private/non-access returns identical `repo.not_found` (D-23–D-25) | ✓ VERIFIED | `resolve_repo_for_read` + `repo_private_404_*` tests |
| 6 | Default-branch rename/delete via intended APIs returns soft-protect error | ✓ VERIFIED | Soft-protect checks in `branch_rename`/`branch_delete`; named test **PASS** — undermined by gap #1 bypass path |
| 7 | Archive/treeish argv cannot be interpreted as git CLI options | ✗ FAILED | `validate_treeish` lacks `starts_with('-')`; `archive` omits `--`; CR-01 reproducible |

**Score:** 6/7 truths verified (0 present behavior-unverified in score set; 3 backstop items below)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/octanest-git/src/cli.rs` | CliGitBackend | ✓ VERIFIED | Substantive (~1267 lines); wired via `AppState.git` — **unsafe argv validation** (gaps) |
| `crates/octanest-git/src/backend.rs` | GitBackend trait | ✓ VERIFIED | Trait + future Gix docs |
| `crates/octanest-git/src/version.rs` | assert_git_version ≥2.5 | ✓ VERIFIED | Called from `main.rs` → exit(1) |
| `crates/octanest-db/migrations/*/0007_repositories.sql` | repos schema | ✓ VERIFIED | sqlite/postgres/mysql present |
| `crates/octanest-api/src/repo/mod.rs` | repo.create + browse/branch | ✓ VERIFIED | Wired to `ctx.git` |
| `crates/octanest-api/src/routes/repo_raw.rs` | raw + archive HTTP | ✓ VERIFIED | Mounted in `app.rs` — archive validation gap |
| `apps/web/src/routes/new.tsrx` | create UI | ✓ VERIFIED | rpc-gen client create |
| `apps/web/src/routes/$owner.$repo.{tree,blob,branches,tags,settings}*` | browse UI | ✓ VERIFIED | Routes exist; branches call branchCreate/Rename/Delete |
| `docs/ARCHITECTURE.md` | GitBackend docs | ✓ VERIFIED | CliGitBackend / GixGitBackend section |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `repo/mod.rs` | `CliGitBackend` / trait | `ctx.git.init_bare` / `ls_tree` / `branch_*` | ✓ WIRED | Dyn trait on RpcCtx — gsd-tools path grep false-negative on `cli.rs` literal |
| `main.rs` | `version.rs` | `assert_git_version` | ✓ WIRED | Import + boot call |
| `new.tsrx` | `repo.create` | apiClient | ✓ WIRED | |
| `branches.tsrx` | `repo.branch_*` | apiClient | ✓ WIRED | |
| `clone-box.tsrx` | archive HTTP | `/api/repos/.../archive/` | ✓ WIRED | |
| `blob.$.tsrx` | `highlight.ts` | Shiki | ✓ WIRED | grammars for tsrx/ripple |
| index README | `markdown.ts` | rehype-sanitize | ✓ WIRED | |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Tree/blob UI | tree entries / blob bytes | `repo.tree` / `repo.blob` → `git.ls_tree` / `cat_blob` | Yes (CLI) | ✓ FLOWING |
| Archive download | zip/tar.gz bytes | `serve_archive` → `git.archive` | Yes | ✓ FLOWING (unsafe argv) |
| Branch list | refs | `list_refs` | Yes | ✓ FLOWING |
| /new create | repo row + bare dir | DB insert + `init_bare` | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Soft-protect default rename/delete | `cargo test -p octanest-api --test repo_branch_soft_protect repo_branch_soft_protect_blocks_default_rename_and_delete -- --exact` | 1 passed | ✓ PASS |
| Archive zip/tar.gz nonempty | `cargo test -p octanest-api --test repo_archive repo_archive_zip_and_tar_gz_nonempty_for_seeded_ref -- --exact` | 1 passed | ✓ PASS |
| CR-02 force-delete via `git branch -D` | Host git: `git branch -D main` as name/start pattern | Deleted branch main | ✗ FAIL (injection works) |
| CR-01 `--output=` archive | Host git archive with option-like argv | Empty file created | ✗ FAIL (injection works) |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| — | — | No phase-declared `scripts/*/tests/probe-*.sh` | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| GIT-01 | 00,02,03,04,09,12,13,16 | Create public/private repo | ✓ SATISFIED | create RPC + /new + migrations |
| GIT-05 | 00,05,06,07,14,15,18 | Browse files/commits/branches/tags | ✓ SATISFIED | browse APIs + Octane routes (WR-03 slashy-ref URL parse is a warning) |
| GIT-06 | 00,07,18 | Branch create/rename/delete where permitted | ✗ BLOCKED | Soft-protect bypass CR-02 |
| GIT-07 | 00,08 | Download source archive | ⚠️ PARTIAL | Happy-path works; CR-01 option injection unfixed |
| GIT-08 | 00,02,05,09,10,12,17 | Filesystem / volume-backed objects | ✓ SATISFIED | repos_dir + Compose volume + bare layout |
| GIT-09 | 00,01,11,12,17 | System git CLI ≥2.5 + CliGitBackend | ✓ SATISFIED | version gate + Cli adapter |
| GIT-10 | 00,01,11,12 | Swappable GitBackend; Gix future docs | ✓ SATISFIED | ARCHITECTURE + trait seam; no gix primary |

Orphaned phase requirements: none (GIT-02..04 are Phase 8/9 — not Phase 7).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `crates/octanest-git/src/cli.rs` | 58–72, 849–903 | Missing leading-`-` reject / missing `--` before refs | 🛑 BLOCKER | CR-01/CR-02 from 07-REVIEW — unfixed; reproducible |
| `crates/octanest-api/src/routes/repo_raw.rs` | 42–76 | `validate_archive_treeish` allows option-like refs; raw `validate_ref` rejects `/` | 🛑 BLOCKER / ⚠️ WARNING | CR-01; WR-02 inconsistency |
| `apps/web/src/lib/repo-browse.ts` | 14–25 | `parseRefAndPath` first-segment-only | ⚠️ WARNING | WR-03 hierarchical branch URLs (D-17) |
| `crates/octanest-api/src/repo/mod.rs` | ~662–709 | DB row left if git init/seed fails | ⚠️ WARNING | WR-01 — name stuck until manual cleanup |

No unreferenced `TBD`/`FIXME`/`XXX` debt markers found in core phase git/repo paths scanned.

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `repo_branch_soft_protect.rs` | GIT-06 | yes | none found | no | Behavioral | OK for intended paths; **missing** `-D` injection case |
| `repo_archive.rs` | GIT-07 | yes | none found | no | Behavioral | OK happy-path; **missing** `--output` injection case |
| `repo_create.rs` / `repo_private_404.rs` | GIT-01/05 | listed | — | no | Behavioral | OK |
| `highlight.test.ts` | GIT-05/D-19 | yes | — | no | Value | OK for grammar id presence |

**Disabled tests on requirements:** 0  
**Circular patterns detected:** 0  
**Insufficient assertions:** injection cases absent → WARNING (feeds gaps)

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts. (38/38 honored; non-blocking)

### Prohibitions (judgment-tier)

| Prohibition | Status | Notes |
| ----------- | ------ | ----- |
| Must not allow delete/rename of default branch from Phase 7 API | ⚠️ unverified-prohibition — human review recommended | Violated via CR-02 `branchCreate("-D")` |
| Must not shell via `sh -c` | ✓ held | argv `Command` arrays only |
| Must not implement GixGitBackend body | ✓ held | docs only |
| Must not return archive bytes via JSON RPC | ✓ held | HTTP streaming |
| Must not reveal private via distinct errors | ✓ held | unified not_found |

### Human Verification Required

Status is `gaps_found` (security blockers). After gap closure, still run:

1. **Clone/download box (07-08)** — Code tab: HTTPS + SSH placeholder + archive items  
2. **Highlight + README sanitize (07-15)** — .ts/.tsrx blob + script-stripped README  
3. **Backstop visual** — .tsrx/.ripple fidelity; /new textarea wrap; long path ellipsis  

### Gaps Summary

Feature surface for Phase 7 largely ships (create, browse, archives, branch UI, FS + CliGitBackend + docs). **Goal is blocked** by unfixed **07-REVIEW critical findings CR-01 and CR-02**: git option injection through unvalidated treeish/branch argv. Soft-protect and safe archive invariants fail closed. Warnings WR-01–WR-03 remain advisory relative to those blockers but should be fixed in the same gap-closure pass where practical.

Structured `gaps:` in frontmatter for `/gsd-plan-phase --gaps`.

---

_Verified: 2026-09-12T19:25:00Z_  
_Verifier: Claude (gsd-verifier)_
