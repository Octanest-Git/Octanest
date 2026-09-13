---
phase: 08-git-https-pats
fixed_at: 2026-09-13T20:19:25Z
review_path: .planning/phases/08-git-https-pats/08-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-09-13T20:19:25Z
**Source review:** `.planning/phases/08-git-https-pats/08-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (Critical + Warning; Info skipped per `critical_warning`)
- Fixed: 4
- Skipped: 0

**Verification environment:** Main checkout (`workflow.use_worktrees=false`). Gates ran in the main working tree, not an isolated worktree.

**Targeted tests:** `cargo nextest run -p octanest-api --locked --test pat_rpc --test git_smart_http` — 18 passed.

## Fixed Issues

### WR-01: Fine-grained PAT create is not transactional

**Files modified:** `crates/octanest-db/src/pats.rs`
**Commit:** `3f416f7`
**Applied fix:** Rewrote `create` so PAT insert + `personal_access_token_repos` links run inside a per-dialect sqlx transaction (`begin` → inserts → `commit`). Removed the non-transactional `insert_repo_links` helper. Link failures roll back the PAT row.

### WR-02: Duplicate `repository_ids` can orphan a fine-grained PAT

**Files modified:** `crates/octanest-api/src/pat/mod.rs`
**Commit:** `69fde82`
**Applied fix:** After ownership checks in `create_fine_grained`, skip pushing a repo id already present in `owned` (order-preserving dedupe) so composite PK collisions cannot fail create after the PAT row exists.

### WR-03: Failed-auth IP identity uses first `X-Forwarded-For` hop (spoofable)

**Files modified:** `crates/octanest-api/src/routes/git_smart_http.rs`, `docs/CONFIGURATION.md`, `docs/API.md`
**Commit:** `57daaba`
**Applied fix:** `client_ip` now takes the rightmost non-empty XFF hop. Documented trusted-proxy / sanitize-forwarded-headers requirement in CONFIGURATION (rate-limit section) and API rate-limits section.

### WR-04: No server-side `expires_at` validation; UI mislabels unparseable expiry

**Files modified:** `crates/octanest-api/src/pat/mod.rs`, `apps/web/src/components/settings/pat-list.tsrx`
**Commit:** `ce30698`
**Status:** `fixed: requires human verification` (logic: RFC3339 + future-instant gate)
**Applied fix:** Added `validate_expires_at` used by classic and fine-grained create (blank → None; invalid RFC3339 / past → `rpc.bad_input`). UI `formatExpiry` maps NaN to `"Invalid expiration"`; `isExpired` treats NaN as expired so the Expired badge shows.

## Skipped Issues

None — all in-scope findings were fixed.

Info findings (IN-01, IN-02, IN-03) were out of scope for this run.

---

_Fixed: 2026-09-13T20:19:25Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
