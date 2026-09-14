---
phase: "11"
slug: "issues"
status: pending
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-14"
updated: "2026-09-14"
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `11-RESEARCH.md` Validation Architecture. Wave 0 gaps closed by plans 00–12.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo nextest (Rust) + Vitest (web) |
| **Config file** | workspace Cargo / `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(issue_)' ; cargo nextest run -p octanest-db -E 'test(dialect_issues) | test(factory_reset_issues)' ; bun --cwd apps/web exec vitest run src/lib/markdown.test.ts src/lib/markdown.issues.test.ts` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~90–240 seconds targeted |
| **Phase gate (11-12)** | Quick run + `cargo test -p octanest-db --lib migration_parity` + `make rpc-sync-check` + `bun --cwd apps/web run build` |

---

## Sampling Rate

- **Per task commit:** focused nextest filter + relevant Vitest file
- **Per wave merge:** `make test` (or nextest workspace + `bun run test` in apps/web)
- **Phase gate:** Full automated gate green + `make rpc-sync-check` before `/gsd-verify-work`

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ISS-01 | create/edit/close/reopen + ACL | API integration | `cargo nextest run -p octanest-api -E 'test(issue_lifecycle)'` | ❌ Wave 0 |
| ISS-01 | per-repo `#N` monotonic / no reuse | dialect + API | `cargo nextest run -p octanest-db -E 'test(dialect_issues)'` | ❌ Wave 0 |
| ISS-01 | Admin hard-delete + confirmNumber | API | `cargo nextest run -p octanest-api -E 'test(issue_delete)'` | ❌ Wave 0 |
| ISS-01 | issue edit history trail | API | `cargo nextest run -p octanest-api -E 'test(issue_history)'` | ❌ Wave 0 |
| ISS-02 | comment CRUD + moderation delete | API | `cargo nextest run -p octanest-api -E 'test(issue_comments)'` | ❌ Wave 0 |
| ISS-02 | comment edit history | API | `cargo nextest run -p octanest-api -E 'test(issue_comments)'` | ❌ Wave 0 |
| ISS-02 | Write\|Preview uses renderGfm sanitize | unit | `bun --cwd apps/web exec vitest run src/lib/markdown.test.ts` | ✅ extend |
| ISS-03 | labels assign Write+ / defs Admin | API | `cargo nextest run -p octanest-api -E 'test(issue_labels)'` | ❌ Wave 0 |
| ISS-03 | assignees Read+ eligibility | API | `cargo nextest run -p octanest-api -E 'test(issue_assignees)'` | ❌ Wave 0 |
| ISS-04 | markdown `#N` / `owner/repo#N` autolink | unit | `bun --cwd apps/web exec vitest run src/lib/markdown.issues.test.ts` | ❌ Wave 0 |
| ISS-04 | link stubs CRUD | API | `cargo nextest run -p octanest-api -E 'test(issue_links)'` | ❌ Wave 0 |
| ISS-* | reactions toggle (issue + comment) | API | `cargo nextest run -p octanest-api -E 'test(issue_reactions)'` | ❌ Wave 0 |
| ISS-01..04 | private ACL soft not-found | API | extend `repo_private_404` / `test(issue_private)` | ❌ Wave 0 |
| UI | Issues tab + list/detail/new routes | web integration | `bun --cwd apps/web exec vitest run src/routes/\$owner.\$repo.issues.integration.test.ts` | ❌ Wave 0 |
| OPS | factory reset cascades issue tables | DB | `cargo nextest run -p octanest-db -E 'test(factory_reset_issues)'` | ❌ Wave 0 |

---

## Wave 0 Gaps

- [ ] `crates/octanest-api/tests/issue_lifecycle.rs` — ISS-01 create/edit/close/reopen + history
- [ ] `crates/octanest-api/tests/issue_delete.rs` — Admin hard-delete + confirmNumber
- [ ] `crates/octanest-api/tests/issue_comments.rs` — ISS-02 comments + moderation + history
- [ ] `crates/octanest-api/tests/issue_labels.rs` — ISS-03 label defs + assign
- [ ] `crates/octanest-api/tests/issue_assignees.rs` — ISS-03 multi-assignee + Read+ eligibility
- [ ] `crates/octanest-api/tests/issue_links.rs` — ISS-04 stubs + manual link
- [ ] `crates/octanest-api/tests/issue_reactions.rs` — D-ISS-11 eight reactions
- [ ] Extend private soft not-found for unauthorized issue access (D-ISS-20)
- [ ] `crates/octanest-db/tests/dialect_issues.rs` — `0011_issues` tri-dialect
- [ ] `crates/octanest-db/tests/factory_reset_issues.rs` — cascade wipe
- [ ] `apps/web/src/routes/$owner.$repo.issues.integration.test.ts` — Issues tab + list/detail/new
- [ ] `apps/web/src/lib/markdown.issues.test.ts` — `#N` / `owner/repo#N` + sanitize regression

---

## Manual / UAT Backstops

- Create issue `#1`; edit title/body; close and reopen; Admin hard-delete with typed number confirm
- Comment thread: author edit/delete; Write+ moderate-delete others; Write\|Preview markdown
- Org default labels + repo hide/local-only; assign labels/assignees with Write+; Admin-only label defs
- Autolink `#N` and `owner/repo#N` in body; Linked PRs panel shows stubs; manual link control
- List: Open default; Closed/All; filter author/label/assignee/text; offset pages
- Private repo: unauthorized viewer gets soft not-found (no issue enumeration)
