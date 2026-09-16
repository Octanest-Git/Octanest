---
phase: "12"
slug: "pull-requests"
status: drafted
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-16"
---

# Phase 12 — Validation Strategy

> Per-phase validation contract. Wave 0 stubs in plans 00–01; implementation plans turn green.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo nextest + Vitest |
| **Quick run** | `cargo nextest run -p octanest-api -E 'test(pull_)' ; cargo nextest run -p octanest-db -E 'test(dialect_pulls) | test(factory_reset_pulls)' ; cargo nextest run -p octanest-git -E 'test(merge_)' ; bun --cwd apps/web exec vitest run src/routes/\$owner.\$repo.pulls.integration.test.ts` |
| **Full suite** | `make test` |
| **Phase gate** | Quick run + `make rpc-sync-check` + `make web-lint` + `make web-format-check` |

## Sampling Rate

- Per task: focused nextest / vitest filter
- Per wave: targeted `pull_` + web pulls tests
- Phase gate: full automated gate before verify-work

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| PR-01 | open same-repo + fork head | API | `cargo nextest run -p octanest-api -E 'test(pull_lifecycle)'` | ❌ Wave 0 |
| PR-02 | diff/commits/conversation | API | `cargo nextest run -p octanest-api -E 'test(pull_files) | test(pull_commits)'` | ❌ |
| PR-03 | general + line comments | API | `cargo nextest run -p octanest-api -E 'test(pull_comments)'` | ❌ |
| PR-04 | approve / request changes | API | `cargo nextest run -p octanest-api -E 'test(pull_reviews)'` | ❌ |
| PR-05 | merge/squash/rebase | API + git | `cargo nextest run -p octanest-api -E 'test(pull_merge)'` ; `cargo nextest run -p octanest-git -E 'test(merge_)'` | ❌ |
| PR-06 | close/reopen | API | `cargo nextest run -p octanest-api -E 'test(pull_lifecycle)'` | ❌ |
| PR-07 | merge strategy settings | API | `cargo nextest run -p octanest-api -E 'test(pull_merge_settings)'` | ❌ |
| PR-* | private soft not-found | API | extend `repo_private_404` | ❌ |
| UI | Pulls tab + routes | web | vitest pulls.integration.test.ts | ❌ |
| OPS | dialect + factory reset | DB | dialect_pulls / factory_reset_pulls | ❌ |

## Wave 0 Gaps

- [ ] `crates/octanest-api/tests/pull_lifecycle.rs`
- [ ] `crates/octanest-api/tests/pull_comments.rs`
- [ ] `crates/octanest-api/tests/pull_reviews.rs`
- [ ] `crates/octanest-api/tests/pull_merge.rs`
- [ ] `crates/octanest-api/tests/pull_merge_settings.rs`
- [ ] `crates/octanest-api/tests/pull_files.rs` (or combined)
- [ ] `crates/octanest-db/tests/dialect_pulls.rs`
- [ ] `crates/octanest-db/tests/factory_reset_pulls.rs`
- [ ] `crates/octanest-git` merge_* tests (with impl plan)
- [ ] `apps/web/src/routes/$owner.$repo.pulls.integration.test.ts`
- [ ] Private ACL pull cases
