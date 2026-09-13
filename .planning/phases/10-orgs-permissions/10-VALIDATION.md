---
phase: "10"
slug: "orgs-permissions"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-14"
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `10-RESEARCH.md` Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo nextest (Rust) + Vitest (web) |
| **Config file** | workspace Cargo / `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(repo_private) or test(acl) or test(org)'` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~90–240 seconds targeted |

---

## Sampling Rate

- **Per task commit:** focused nextest filter + relevant Vitest file
- **Per wave merge:** `make test`
- **Phase gate:** Full suite green before `/gsd-verify-work`; dialect migration smoke if 0009 touches MySQL quirks

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ORG-01 | Create org; add member by username; email invite accept | API integration | `cargo nextest run -p octanest-api -E 'test(org_)'` | ❌ Wave 0 |
| ORG-02 | Owner/Admin admin; Member respects member_base none/read/write | unit + API | colocated `acl` unit + org ACL integration | ❌ Wave 0 |
| ORG-03 | Collaborator CRUD on personal + org repos; visibility admin-gated | API + Vitest | extend repo settings + UI test | ❌ Wave 0 |
| ORG-04 | Private non-grantee → web `repo.not_found`; git → 401; push denied without write | API | extend `repo_private_404.rs`, `git_smart_http.rs` | ✅ extend |
| ORG-04 | PAT collaborator push with classic `repo` scope | API | extend `pat_rpc` / smart http | ✅ partial |
| ORG-01 | Username lookup rate/limit shape | API unit | new | ❌ Wave 0 |
| ORG-01/03 | `/orgs/new`, owner picker, collaborators UI | Vitest | new route tests | ❌ Wave 0 |

---

## Wave 0 Gaps

- [ ] `crates/octanest-api/tests/org_create_members.rs` — ORG-01/02
- [ ] `crates/octanest-api/tests/org_invites.rs` — email invite + closed signup
- [ ] `crates/octanest-api/tests/repo_collaborators_acl.rs` — ORG-03/04 matrix
- [ ] Extend `repo_private_404.rs` + `git_smart_http.rs` for collaborator/org Member cases
- [ ] Extend PAT authorize tests for non-owner collaborator
- [ ] `apps/web` integration tests for `/orgs/new`, owner picker, collaborators panel
- [ ] Factory reset coverage for org tables
- [ ] Unit tests in `acl.rs` for coalesce matrix

---

## Manual / UAT Backstops

- Create org; add member via live username lookup; email invite accept
- member_base_permission none/read/write behavior on private org repo
- Collaborator read/write/admin on personal and org-owned repo
- Unauthorized cannot see private repo in UI; git gets 401
