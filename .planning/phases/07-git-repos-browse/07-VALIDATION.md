---
phase: "07"
slug: "git-repos-browse"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-12"
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: cargo nextest + `cargo test`; Web: Vitest 5 (unit / integration / e2e) |
| **Config file** | `apps/web/vitest.config.ts`; Makefile `make test` |
| **Quick run command** | `cargo nextest run -p octanest-git --lib` && `cd apps/web && bun run test:unit` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run targeted nextest filter + relevant Vitest project
- **After every plan wave:** Run `make test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 07-W0 | 00 | 0 | GIT-* | — | N/A | stubs | Wave 0 scaffolding | ❌ W0 | ⬜ pending |
| GIT-01 | TBD | TBD | GIT-01 | T-07-auth | verified-only create; duplicate inline error | integration | `cargo nextest run -p octanest-api -- repo_create` | ❌ W0 | ⬜ pending |
| GIT-05 | TBD | TBD | GIT-05 | T-07-acl | private non-owner → not_found | unit/integration | `cargo nextest run -p octanest-git` / `repo_private_404` | ❌ W0 | ⬜ pending |
| GIT-06 | TBD | TBD | GIT-06 | T-07-branch | owner CRUD; default soft-protect | integration | `cargo nextest run -p octanest-api -- repo_branch_soft_protect` | ❌ W0 | ⬜ pending |
| GIT-07 | TBD | TBD | GIT-07 | — | zip + tar.gz for ref | unit | `cargo nextest run -p octanest-git -- git_archive_formats` | ❌ W0 | ⬜ pending |
| GIT-08 | TBD | TBD | GIT-08 | T-07-path | files under repos_dir owner/name.git | integration | `cargo nextest run -p octanest-api -- repo_fs_layout` | ❌ W0 | ⬜ pending |
| GIT-09/10 | TBD | TBD | GIT-09, GIT-10 | — | CliGitBackend + trait/docs | unit | `cargo test -p octanest-git` | ❌ W0 | ⬜ pending |
| D-33 | TBD | TBD | GIT-09 | — | boot rejects missing/old git | unit | `cargo nextest run -p octanest-git -- git_version_gate` | ❌ W0 | ⬜ pending |
| UI | TBD | TBD | GIT-01 | — | `/new` wall; home CTA | integration | `bun --cwd apps/web run test:integration` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-git` crate + version gate unit tests
- [ ] `crates/octanest-api/tests/repo_*.rs` integration harness (temp repos_dir + DB)
- [ ] Tri-dialect migration `0007_repositories` (+ account default_branch / instance default_visibility as needed)
- [ ] Web integration tests for SignedInHome CTA → `/new` and unverified wall
- [ ] Dockerfile installs `git`; Compose volume for `var/repos`
- [ ] Extend reserved username list with `"new"` (and other flat routes)
- [ ] Docs: amend GIT-09 wording; ARCHITECTURE GitBackend section

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Syntax highlighting for `.tsrx` / `.ripple` + common languages | GIT-05 / UI-SPEC | Visual fidelity | Open blob views for sample files; confirm highlight + line permalinks |
| Safe Markdown README render | GIT-05 / D-18 | Sanitizer edge cases | Render README with HTML/script attempts; confirm stripped |
| Clone/download box UX (HTTPS shown, SSH placeholder) | D-22 | UI copy | Inspect Code tab clone box on a seeded repo |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
