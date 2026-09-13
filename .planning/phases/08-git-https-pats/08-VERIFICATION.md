---
phase: 08-git-https-pats
verified: 2026-09-13T21:16:36Z
status: passed
score: 7/7 must-haves verified
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/phases/08-git-https-pats/08-00-PLAN.md
  - .planning/phases/08-git-https-pats/08-00-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-01-PLAN.md
  - .planning/phases/08-git-https-pats/08-01-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-02-PLAN.md
  - .planning/phases/08-git-https-pats/08-02-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-03-PLAN.md
  - .planning/phases/08-git-https-pats/08-03-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-04-PLAN.md
  - .planning/phases/08-git-https-pats/08-04-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-05-PLAN.md
  - .planning/phases/08-git-https-pats/08-05-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-06-PLAN.md
  - .planning/phases/08-git-https-pats/08-06-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-07-PLAN.md
  - .planning/phases/08-git-https-pats/08-07-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-08-PLAN.md
  - .planning/phases/08-git-https-pats/08-08-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-09-PLAN.md
  - .planning/phases/08-git-https-pats/08-09-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-10-PLAN.md
  - .planning/phases/08-git-https-pats/08-10-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-11-PLAN.md
  - .planning/phases/08-git-https-pats/08-11-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-12-PLAN.md
  - .planning/phases/08-git-https-pats/08-12-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-13-PLAN.md
  - .planning/phases/08-git-https-pats/08-13-SUMMARY.md
  - .planning/phases/08-git-https-pats/08-CONTEXT.md
  - .planning/phases/08-git-https-pats/08-DISCUSSION-LOG.md
  - .planning/phases/08-git-https-pats/08-REVIEW-FIX.md
  - .planning/phases/08-git-https-pats/08-REVIEW.md
  - .planning/phases/08-git-https-pats/08-SECURITY.md
  - .planning/phases/08-git-https-pats/08-UAT.md
  - .planning/phases/08-git-https-pats/08-VALIDATION.md
  - Makefile
  - apps/web/src/components/chrome.tsrx
  - apps/web/src/components/repo/clone-box.pat.integration.test.ts
  - apps/web/src/components/repo/clone-box.tsrx
  - apps/web/src/components/repo/pat-how-to.tsrx
  - apps/web/src/components/repo/quick-setup.tsrx
  - apps/web/src/components/settings/pat-classic-form.tsrx
  - apps/web/src/components/settings/pat-fg-form.tsrx
  - apps/web/src/components/settings/pat-list.tsrx
  - apps/web/src/components/settings/pat-reveal.tsrx
  - apps/web/src/components/settings/pat-revoke-dialog.tsrx
  - apps/web/src/components/settings/settings-nav.tsrx
  - apps/web/src/routes/settings/profile.tsrx
  - apps/web/src/routes/settings/tokens.integration.test.ts
  - apps/web/src/routes/settings/tokens.new.fine-grained.tsrx
  - apps/web/src/routes/settings/tokens.new.tsrx
  - apps/web/src/routes/settings/tokens.tsrx
  - crates/octanest-api/src/app.rs
  - crates/octanest-api/src/auth/session.rs
  - crates/octanest-api/src/git/http_backend.rs
  - crates/octanest-api/src/git/mod.rs
  - crates/octanest-api/src/pat/mod.rs
  - crates/octanest-api/src/pat/rate_limit.rs
  - crates/octanest-api/src/repo/acl.rs
  - crates/octanest-api/src/routes/git_smart_http.rs
  - crates/octanest-api/src/routes/mod.rs
  - crates/octanest-api/src/rpc.rs
  - crates/octanest-api/tests/git_smart_http.rs
  - crates/octanest-api/tests/pat_rpc.rs
  - crates/octanest-core/src/auth_types.rs
  - crates/octanest-core/src/lib.rs
  - crates/octanest-core/src/pat_types.rs
  - crates/octanest-db/migrations/mysql/0008_pats.sql
  - crates/octanest-db/migrations/postgres/0008_pats.sql
  - crates/octanest-db/migrations/sqlite/0008_pats.sql
  - crates/octanest-db/src/lib.rs
  - crates/octanest-db/src/pats.rs
  - crates/octanest-db/tests/dialect_pats.rs
  - docker-compose.yml
  - docs/API.md
  - docs/ARCHITECTURE.md
  - docs/CONFIGURATION.md
  - packages/api-client/src/index.ts
  - scripts/smoke-git-https.sh
covered_digest: "v1:sha256:b9ec553452fa85192f824cd3357d8dbf640f5358de5cf4aa4462027509dd4731"
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 26
  total: 26
  not_honored: []
re_verification:
  previous_status: human_needed
  previous_score: 6/7
  gaps_closed:
    - "08-UAT.md complete 4/4 — tokens UI, FG+Smart HTTP, how-to wrap backstop, judgment prohibitions"
    - "Truth 7 long clone URL wrap — upgraded from insufficient_spec via UAT #3 + vitest wrap-class assertion"
    - "Post-review fixes WR-01..04, IN-01..03 still present (transactional FG create, XFF rightmost, expires_at, repos_required, token_prefix fingerprint, cookie+private 401)"
  gaps_remaining: []
  regressions: []
behavior_unverified_items: []
human_verification: []
---

# Phase 8: Git HTTPS & PATs Verification Report

**Phase Goal:** Users can authenticate git over HTTPS with personal access tokens (never account passwords) and manage those tokens in the UI

**Verified:** 2026-09-13T21:16:36Z  
**Status:** passed  
**Re-verification:** Yes — after UAT 4/4 + code-review fixes (WR-01..04, IN-01..03)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | User can create, list, and revoke personal access tokens for HTTPS git (session RPC; PATs not RPC Bearer per D-01 / GIT-11 “where applicable”) | ✓ VERIFIED | `pat.createClassic` / `createFineGrained` / `list` / `revoke` in `crates/octanest-api/src/pat/mod.rs` + `rpc.rs`; `pat_create_classic_returns_one_time_token`, `pat_revoke_removes_from_list` PASS this run |
| 2 | User can clone, fetch, and push over HTTPS using a PAT; account password is rejected for git auth | ✓ VERIFIED | Smart HTTP Basic → SHA-256 → `find_pat_by_token_hash`; `looks_like_pat` rejects non-prefix secrets; `git_smart_pat_push_fetch_happy_path`, `git_smart_basic_account_password_rejected_401` PASS |
| 3 | Token prefixes are `octanest_pat_` / `octanest_fg_` (D-08 locked) | ✓ VERIFIED | `CLASSIC_PAT_PREFIX` / `FINE_GRAINED_PAT_PREFIX` in `pat_types.rs`; unit asserts reject `ona_*` / github prefixes; mint stores brand + 8-hex fingerprint (IN-02) |
| 4 | HTTPS Smart HTTP URL is `/{owner}/{repo}.git` (D-18) | ✓ VERIFIED | Axum `/{owner}/{repo_git}/…`; Traefik `PathRegexp(^/[^/]+/[^/]+\.git)` priority 110 in `docker-compose.yml` |
| 5 | Private unauth git → 401 + `WWW-Authenticate`; PATs HTTPS-git-only (not RPC Bearer) (D-21 / D-01) | ✓ VERIFIED | `git_smart_private_anon_401_www_authenticate` PASS; `git_smart_session_cookie_ignored_as_anon` covers private+cookie → 401 (IN-03); docs/API D-01 session-only RPC |
| 6 | Users manage tokens in UI (list/create classic+FG/revoke + CloneBox how-to) | ✓ VERIFIED | Routes `/settings/tokens`, `/new`, `/new/fine-grained`; vitest tokens + clone-box **17/17 PASS** this run |
| 7 | Long clone URLs wrap or overflow-x-auto in how-to code blocks | ✓ VERIFIED | `pat-how-to.tsrx` `overflow-x-auto whitespace-pre-wrap break-all`; vitest asserts those classes; **08-UAT.md test 3 pass** closes prior backstop abstain |

**Score:** 7/7 truths verified (0 present, behavior-unverified)

**Locked decisions honored in code (not SUMMARY claims):**

- Prefixes: `octanest_pat_` / `octanest_fg_` (not `ona_*`)
- Clone path: `/{owner}/{repo}.git`
- Private unauth: 401 + WWW-Authenticate; PATs not RPC Bearer

### Advisory (New Scope, Unevidenced)

None — re-verification after UAT/review; no new unevidenced Step 7 blockers.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/octanest-db/migrations/*/0008_pats.sql` | PAT tables + `token_hash` | ✓ VERIFIED | Tri-dialect; hash UNIQUE; no plaintext column |
| `crates/octanest-db/src/pats.rs` | create/find/list/revoke | ✓ VERIFIED | Transactional FG create (WR-01); wired via `Database` |
| `crates/octanest-core/src/pat_types.rs` | prefixes + DTOs | ✓ VERIFIED | Classic/FG kinds; list omits secret |
| `crates/octanest-api/src/pat/mod.rs` | PAT RPC | ✓ VERIFIED | createClassic/FG, list, revoke; `pat.repos_required` (IN-01); `validate_expires_at` (WR-04) |
| `crates/octanest-api/src/routes/git_smart_http.rs` | Smart HTTP auth | ✓ VERIFIED | Basic PAT, ACL, rate limit, rightmost XFF (WR-03), cookie ignore |
| `crates/octanest-api/src/git/http_backend.rs` | CGI helper | ✓ VERIFIED | Exists + used by Smart HTTP |
| `apps/web/src/routes/settings/tokens*.tsrx` | Token UI | ✓ VERIFIED | List + classic + FG create |
| `apps/web/src/components/repo/pat-how-to.tsrx` | HTTPS how-to | ✓ VERIFIED | Wired into CloneBox + QuickSetup; wrap classes asserted |
| `packages/api-client/src/index.ts` | Generated client | ✓ VERIFIED | `make rpc-sync-check` ok |
| `docker-compose.yml` + `scripts/smoke-git-https.sh` | Traefik + smoke | ✓ VERIFIED | PathRegexp + smoke target present |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `git_smart_http.rs` | DB `find_pat_by_token_hash` | Basic password → SHA-256 | ✓ WIRED | |
| `pat/mod.rs` | `octanest_db` pats | `create_pat` / `list_pats_for_user` / `revoke_pat` | ✓ WIRED | |
| `tokens.tsrx` / `pat-list.tsrx` | api-client | `patListQueryOptions` / `apiClient.pat.*` | ✓ WIRED | |
| `tokens.new*.tsrx` | api-client | `createClassic` / `createFineGrained` | ✓ WIRED | |
| `clone-box.tsrx` | `pat-how-to.tsrx` | `<PatHowTo />` | ✓ WIRED | Also QuickSetup |
| `app.rs` | Smart HTTP handlers | `/{owner}/{repo_git}/…` | ✓ WIRED | |
| `docker-compose.yml` | API Smart HTTP | Traefik PathRegexp | ✓ WIRED | |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Pat list UI | `list.data` | `pat.list` RPC → `list_pats_for_user` | Yes (Query) | ✓ FLOWING |
| Classic create | `plaintext` | `create_classic` mint → one-time response | Yes | ✓ FLOWING |
| Smart HTTP auth | `pat` row | `find_pat_by_token_hash` | Yes | ✓ FLOWING |
| How-to clone example | `httpsUrl` prop | Repo clone URL from parent | Yes (caller-supplied) | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Password rejected | `cargo test -p octanest-api --test git_smart_http git_smart_basic_account_password_rejected_401 -- --exact` | ok | ✓ PASS |
| Create classic | `… pat_rpc pat_create_classic_returns_one_time_token` | ok | ✓ PASS |
| Push/fetch PAT | `… git_smart_pat_push_fetch_happy_path` | ok | ✓ PASS |
| Private 401 WWW-Auth | `… git_smart_private_anon_401_www_authenticate` | ok | ✓ PASS |
| Cookie ignored (private) | `… git_smart_session_cookie_ignored_as_anon` | ok | ✓ PASS |
| Revoke | `… pat_revoke_removes_from_list` | ok | ✓ PASS |
| Tokens + how-to UI | `vitest run tokens.integration.test.ts clone-box.pat.integration.test.ts` | 17 passed | ✓ PASS |
| Client sync | `make rpc-sync-check` | ok | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| `scripts/smoke-git-https.sh` | (not re-run live) | Documented skip without Docker; UAT #2 accepted nextest + smoke script presence | ⚠️ SKIP (env; covered by UAT automated map) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| **GIT-02** | 00–08, 12–13 | Clone/fetch/push HTTPS with PAT; not account password | ✓ SATISFIED | Smart HTTP + password-reject + push/fetch tests; UI how-to; UAT #2/#3 |
| **GIT-11** | 00–06, 08–11, 13 | Create/list/revoke PATs for HTTPS git (RPC/API where applicable) | ✓ SATISFIED | PAT RPC + UI; D-01 HTTPS-git-only; management via session RPC; UAT #1/#4 |

Orphaned requirements mapped to Phase 8: none (only GIT-02, GIT-11). Both marked Complete in REQUIREMENTS.md.

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts (26/26). Message: *All trackable CONTEXT.md decisions are honored by shipped artifacts.*

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `crates/octanest-api/tests/pat_rpc.rs` | GIT-11 | 10 | 0 | No | Behavioral / value | PASS |
| `crates/octanest-api/tests/git_smart_http.rs` | GIT-02 | 8 | 0 | No | Behavioral / status | PASS |
| `crates/octanest-db/tests/dialect_pats.rs` | GIT-11 | 2 | 0 | No | Value (schema) | PASS |
| `apps/web/.../tokens.integration.test.ts` | GIT-11 | (suite) | 0 | No | Behavioral (mocked RPC) | PASS |
| `apps/web/.../clone-box.pat.integration.test.ts` | GIT-02 | (suite) | 0 | No | Value (copy/CTA/wrap) | PASS |

**Disabled tests on requirements:** 0  
**Circular patterns detected:** 0  
**Insufficient assertions:** 0 blockers

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TBD/FIXME/XXX debt markers in key PAT/Smart HTTP/UI sources | — | — |

### Human Verification Required

N/A — `08-UAT.md` status `complete`, **4/4 passed** (automated evidence map). Prior human_verification items closed; no new blocking gaps found. SECURITY.md `threats_open: 0`.

### Gaps Summary

None. Roadmap success criteria, GIT-02 / GIT-11, and all 7 must-have truths are verified in codebase with named tests + UAT closure. Post-review fixes remain wired. Phase goal achieved.

### Inversion / disconfirmation notes

1. **GIT-11 “RPC/API where applicable”** — Intentional Phase 8 narrowing (D-01): PATs are HTTPS-git-only; token CRUD uses session RPC. Not a gap.  
2. **Web vitest mocks** — Do not alone prove RPC; nextest `pat_rpc` / `git_smart_http` close that gap.  
3. **Compose smoke** — Live Docker path still optional; UAT accepted nextest + script presence.

---

_Verified: 2026-09-13T21:16:36Z_  
_Verifier: Claude (gsd-verifier)_
