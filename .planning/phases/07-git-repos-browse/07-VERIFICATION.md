---
phase: 07-git-repos-browse
verified: 2026-09-12T20:01:51Z
status: passed
score: 7/7 must-haves verified
next_action: "Human verification required. Complete the manual tests in the phase's *-UAT.md, then re-run the verify step until status is passed."
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
  - .planning/phases/07-git-repos-browse/07-19-PLAN.md
  - .planning/phases/07-git-repos-browse/07-19-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-20-PLAN.md
  - .planning/phases/07-git-repos-browse/07-20-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-21-PLAN.md
  - .planning/phases/07-git-repos-browse/07-21-SUMMARY.md
  - .planning/phases/07-git-repos-browse/07-REVIEW.md
  - apps/web/src/components/repo/clone-box.tsrx
  - apps/web/src/lib/highlight.ts
  - apps/web/src/lib/markdown.ts
  - apps/web/src/lib/repo-browse.ts
  - apps/web/src/lib/repo-browse.unit.test.ts
  - apps/web/src/routes/$owner.$repo.blame.$.tsrx
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
  - crates/octanest-api/tests/repo_archive.rs
  - crates/octanest-api/tests/repo_branch_soft_protect.rs
  - crates/octanest-api/tests/repo_create.rs
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

covered_digest: "v1:sha256:c949dba2bda73010d9cb3528d41b5a95e2bd4e2083560653e45fc4b444661a4d"
behavior_unverified: 3
overrides_applied: 0
decision_coverage:
  honored: 38
  total: 38
  not_honored: []
re_verification:
  previous_status: gaps_found
  previous_score: 6/7
  gaps_closed:
    - "User can create, rename, and delete branches from the web UI where permitted (CR-02 / soft-protect integrity) — closed by 07-19"
    - "Archive/treeish argv cannot be interpreted as git CLI options (CR-01) — closed by 07-20"
    - "WR-01 create compensate soft-delete — closed by 07-21"
    - "WR-02 raw slash parity — closed by 07-20"
    - "WR-03 hierarchical parseRefAndPath — closed by 07-21"
  gaps_remaining: []
  regressions: []
advisory: []
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
human_verification:

  - test: "Open a seeded blob for .tsrx and .ripple in the Code UI"
    expected: "Tokens highlight via in-repo grammars (not plain TS/JS alias look)"
    why_human: "verification: backstop — presence of grammars/tests does not prove visual fidelity"
  - test: "Paste a long unbroken description on /new"
    expected: "Text wraps; no horizontal page overflow"
    why_human: "verification: backstop — layout cannot be proven by grep"
  - test: "Browse a deep/long file path in tree/blob chrome"
    expected: "Ellipsis or wrap per UI-SPEC; layout remains usable"
    why_human: "verification: backstop — visual layout only"
  - test: "On Code tab, confirm HTTPS shown, SSH placeholder, archive menu items present (07-08)"
    expected: "Clone box shows HTTPS + SSH placeholder + archive download items"
    why_human: "Harvested <human-check> from 07-08-PLAN — visual chrome"
  - test: "Open a seeded repo blob for .ts / .tsrx; confirm highlight + README sanitize (script attempt stripped) (07-15)"
    expected: "Highlight works; script tags stripped from README render"
    why_human: "Harvested <human-check> from 07-15-PLAN — visual + sanitize behavior in browser"
---

# Phase 7: Git Repos & Browse Verification Report

**Phase Goal:** Users can create filesystem-backed repos and browse history in the UI via system `git` CLI behind a `GitBackend` seam, with a documented future gitoxide path

**Verified:** 2026-09-12T20:01:51Z  
**Status:** human_needed  
**Re-verification:** Yes — after gap closure (07-19, 07-20, 07-21)  
**Next action:** Human verification required. Complete the manual tests in the phase's `*-UAT.md`, then re-run the verify step until status is passed.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Authenticated (and verified, on cloud) user can create a public or private repository | ✓ VERIFIED | `repo.create` + `/new`; WR-01 compensate: `compensate_failed_create` + `repo_create_git_failure_soft_deletes_row_allows_recreate` **PASS** |
| 2 | User can browse files, commits, branches, and tags in the web UI and download a source archive for a ref | ✓ VERIFIED | Routes tree/blob/commits/branches/tags; `clone-box` archives; `repo_archive_zip_and_tar_gz_nonempty_for_seeded_ref` **PASS**; WR-03 longest-prefix parse wired in tree/blob/blame |
| 3 | User can create, rename, and delete branches from the web UI where permitted | ✓ VERIFIED | Soft-protect + CR-02 closed: `reject_option_like_branch`, `validate_treeish` leading-`-`, `branch_*` argv `--`; `repo_branch_create_rejects_option_like_name_leaves_default_intact` **PASS**; `repo_branch_soft_protect_blocks_default_rename_and_delete` **PASS** |
| 4 | Repository objects live on the local filesystem (volume-backed), and git ops use system `git` CLI with docs allowing future gitoxide swap | ✓ VERIFIED | `OCTANEST_REPOS_DIR` + Compose volume; `CliGitBackend`; `assert_git_version` fail-boot; `docs/ARCHITECTURE.md` Cli now / Gix later; no `GixGitBackend` impl body |
| 5 | Private/non-access returns identical `repo.not_found` (D-23–D-25) | ✓ VERIFIED | `resolve_repo_for_read` + `repo_private_404_*` tests present |
| 6 | Default-branch rename/delete via intended APIs returns soft-protect error | ✓ VERIFIED | Soft-protect checks + named test **PASS**; CR-02 bypass path closed (injection regression also **PASS**) |
| 7 | Archive/treeish argv cannot be interpreted as git CLI options | ✓ VERIFIED | `validate_treeish` / `validate_archive_treeish` / `validate_ref` reject leading `-`; `archive` uses `--` before treeish; `repo_archive_rejects_option_like_treeish_no_output_file` **PASS** |

**Score:** 7/7 truths verified (3 backstop items present, behavior-unverified — see Human Verification)

### Deferred Items

None.

### Advisory (New Scope, Unevidenced)

None — re-verification Step 7 found no new-scope unevidenced blockers. Prior WR-01/WR-02/WR-03 advisories were closed by 07-20/07-21.

### Gap Closure Status (prior CR/WR)

| ID | Prior | Now | Evidence |
| ---- | ----- | --- | -------- |
| CR-02 | FAILED (branchCreate `-D` bypass) | **CLOSED** | 07-19; injection test **PASS**; `--` on create/rename/delete |
| CR-01 | FAILED (`--output=` archive write) | **CLOSED** | 07-20; injection test **PASS**; archive `--` + HTTP leading-`-` |
| WR-01 | Advisory (orphan name lock) | **CLOSED** | 07-21; recreate test **PASS**; `compensate_failed_create` |
| WR-02 | Advisory (raw `/` ban) | **CLOSED** | 07-20; `validate_ref` allows `/`, rejects leading `-`; unit tests in `repo_raw.rs` |
| WR-03 | Advisory (first-segment-only parse) | **CLOSED** | 07-21; longest-prefix + vitest 6/6; knownRefs in tree/blob/blame |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/octanest-git/src/cli.rs` | CliGitBackend + safe argv | ✓ VERIFIED | Leading-`-` in `validate_treeish`; `--` on branch_* and archive |
| `crates/octanest-git/src/backend.rs` | GitBackend trait | ✓ VERIFIED | Trait + future Gix docs |
| `crates/octanest-git/src/version.rs` | assert_git_version ≥2.5 | ✓ VERIFIED | Called from `main.rs` → exit(1) |
| `crates/octanest-db/migrations/*/0007_repositories.sql` | repos schema | ✓ VERIFIED | sqlite/postgres/mysql present |
| `crates/octanest-api/src/repo/mod.rs` | create + branch + compensate | ✓ VERIFIED | `reject_option_like_branch`; `compensate_failed_create` |
| `crates/octanest-api/src/routes/repo_raw.rs` | raw + archive HTTP | ✓ VERIFIED | Leading-`-` reject; slashy refs allowed (WR-02) |
| `apps/web/src/lib/repo-browse.ts` | hierarchical parse | ✓ VERIFIED | `parseRefAndPath(splat, knownRefs?)` longest-prefix |
| `apps/web/src/lib/repo-browse.unit.test.ts` | WR-03 unit coverage | ✓ VERIFIED | 6 tests **PASS** |
| `apps/web/src/routes/new.tsrx` | create UI | ✓ VERIFIED | rpc-gen client create |
| `apps/web/src/routes/$owner.$repo.{tree,blob,blame,branches,tags,settings}*` | browse UI | ✓ VERIFIED | knownRefs wired on tree/blob/blame |
| `docs/ARCHITECTURE.md` | GitBackend docs | ✓ VERIFIED | CliGitBackend / GixGitBackend section |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `repo/mod.rs` | `CliGitBackend` | `ctx.git.branch_*` / `init_bare` | ✓ WIRED | Dyn trait on RpcCtx |
| `repo/mod.rs` | `soft_delete_repository` | `compensate_failed_create` | ✓ WIRED | WR-01 on init/seed Err |
| `main.rs` | `version.rs` | `assert_git_version` | ✓ WIRED | Boot call |
| `new.tsrx` | `repo.create` | apiClient | ✓ WIRED | |
| `branches.tsrx` | `repo.branch_*` | apiClient | ✓ WIRED | |
| `clone-box.tsrx` | archive HTTP | `/api/repos/.../archive/` | ✓ WIRED | |
| `repo_raw.rs` | `git.archive` | `serve_archive` | ✓ WIRED | CR-01 hardened |
| `tree/blob/blame.$.tsrx` | `repo-browse.ts` | `parseRefAndPath(..., knownRefs)` | ✓ WIRED | WR-03 / D-17 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Tree/blob UI | tree entries / blob bytes | `repo.tree` / `repo.blob` → CLI | Yes | ✓ FLOWING |
| Archive download | zip/tar.gz bytes | `serve_archive` → `git.archive` | Yes (safe argv) | ✓ FLOWING |
| Branch list | refs | `list_refs` | Yes | ✓ FLOWING |
| /new create | repo row + bare dir | DB insert + `init_bare` (+ compensate) | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| CR-02 injection closed | `cargo test -p octanest-api --test repo_branch_soft_protect repo_branch_create_rejects_option_like_name_leaves_default_intact -- --exact` | 1 passed | ✓ PASS |
| Soft-protect default rename/delete | `cargo test -p octanest-api --test repo_branch_soft_protect repo_branch_soft_protect_blocks_default_rename_and_delete -- --exact` | 1 passed | ✓ PASS |
| CR-01 `--output=` rejected | `cargo test -p octanest-api --test repo_archive repo_archive_rejects_option_like_treeish_no_output_file -- --exact` | 1 passed | ✓ PASS |
| Archive zip/tar.gz nonempty | `cargo test -p octanest-api --test repo_archive repo_archive_zip_and_tar_gz_nonempty_for_seeded_ref -- --exact` | 1 passed | ✓ PASS |
| WR-01 create compensate | `cargo test -p octanest-api --test repo_create repo_create_git_failure_soft_deletes_row_allows_recreate -- --exact` | 1 passed | ✓ PASS |
| WR-03 parseRefAndPath | `bunx vitest run src/lib/repo-browse.unit.test.ts` (apps/web) | 6 passed | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| — | — | No phase-declared `scripts/*/tests/probe-*.sh` | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| GIT-01 | 00,02,03,04,09,12,13,16,21 | Create public/private repo | ✓ SATISFIED | create RPC + /new + WR-01 compensate |
| GIT-05 | 00,05,06,07,14,15,18,20,21 | Browse files/commits/branches/tags | ✓ SATISFIED | browse APIs + routes + WR-02/WR-03 |
| GIT-06 | 00,07,18,19 | Branch create/rename/delete where permitted | ✓ SATISFIED | Soft-protect + CR-02 injection closed |
| GIT-07 | 00,08,20 | Download source archive | ✓ SATISFIED | Happy-path + CR-01 injection closed |
| GIT-08 | 00,02,05,09,10,12,17 | Filesystem / volume-backed objects | ✓ SATISFIED | repos_dir + Compose volume + bare layout |
| GIT-09 | 00,01,11,12,17 | System git CLI ≥2.5 + CliGitBackend | ✓ SATISFIED | version gate + Cli adapter |
| GIT-10 | 00,01,11,12 | Swappable GitBackend; Gix future docs | ✓ SATISFIED | ARCHITECTURE + trait seam; no gix primary |

Orphaned phase requirements: none (GIT-02..04 are Phase 8/9 — not Phase 7).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No unreferenced `TBD`/`FIXME`/`XXX` in gap-closure paths | — | — |
| — | — | Prior CR-01/CR-02 / WR-01..03 patterns **resolved** | — | Closed by 07-19..21 |

No self-evidencing debt markers; no new-scope unevidenced blockers (advisory list empty).

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `repo_branch_soft_protect.rs` | GIT-06 | yes | none | no | Behavioral | OK — includes CR-02 injection case |
| `repo_archive.rs` | GIT-07 | yes | none | no | Behavioral | OK — includes CR-01 `--output` case |
| `repo_create.rs` | GIT-01 | yes | none | no | Behavioral | OK — includes WR-01 compensate |
| `repo-browse.unit.test.ts` | GIT-05 | yes | none | no | Value | OK — hierarchical + fallback |
| `repo_raw.rs` validate_ref_tests | GIT-05 | yes | none | no | Value | OK — WR-02 slash + leading `-` |

**Disabled tests on requirements:** 0  
**Circular patterns detected:** 0  
**Insufficient assertions:** 0 (injection regressions now present)

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts. (38/38 honored; non-blocking)

### Prohibitions

| Prohibition | Status | Notes |
| ----------- | ------ | ----- |
| Must not allow delete/rename of default branch from Phase 7 API | ✓ held | Soft-protect + CR-02 injection regression **PASS** |
| Must not allow user treeish as git CLI options | ✓ held | CR-01 archive injection regression **PASS** |
| Must not shell via `sh -c` | ✓ held | argv `Command` arrays only |
| Must not implement GixGitBackend body | ✓ held | docs only |
| Must not return archive bytes via JSON RPC | ✓ held | HTTP streaming |
| Must not reveal private via distinct errors | ✓ held | unified not_found |

### Human Verification Required

Automated must-haves are green. Complete these before marking the phase fully passed:

### 1. Syntax highlighting fidelity (07-00 backstop)

**Test:** Open a seeded blob for `.tsrx` and `.ripple` in the Code UI  
**Expected:** Tokens highlight via in-repo grammars (not plain TS/JS alias look)  
**Why human:** `verification: backstop` — grammar presence ≠ visual fidelity

### 2. /new description wrap (07-03 backstop)

**Test:** Paste a long unbroken description on `/new`  
**Expected:** Text wraps; no horizontal page overflow  
**Why human:** Layout cannot be proven by grep

### 3. Long path ellipsis (07-15 backstop)

**Test:** Browse a deep/long file path in tree/blob chrome  
**Expected:** Ellipsis or wrap per UI-SPEC; layout remains usable  
**Why human:** Visual layout only

### 4. Clone/download box (07-08)

**Test:** On Code tab, confirm HTTPS shown, SSH placeholder, archive menu items present  
**Expected:** Clone box shows HTTPS + SSH placeholder + archive items  
**Why human:** Harvested planner human-check — visual chrome

### 5. Highlight + README sanitize (07-15)

**Test:** Open a seeded repo blob for `.ts` / `.tsrx`; confirm highlight + README sanitize (script attempt stripped)  
**Expected:** Highlight works; script tags stripped from README render  
**Why human:** Harvested planner human-check — browser behavior

### Gaps Summary

Prior **CR-01** and **CR-02** blockers are closed with green injection regressions (07-19, 07-20). Advisory **WR-01**, **WR-02**, and **WR-03** are closed (07-20, 07-21). All seven roadmap/must-have truths verify in code and named tests. Phase status is **human_needed** solely for end-of-phase visual/backstop UAT — not for security or feature gaps.

---

_Verified: 2026-09-12T20:01:51Z_  
_Verifier: Claude (gsd-verifier)_
