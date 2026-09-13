---
phase: 08-git-https-pats
verified: 2026-09-13T20:06:42Z
status: passed
score: 6/7 must-haves verified
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

covered_digest: "v1:sha256:7acb829a3467948128e08a7a9064bd82e43e71cfc089bca04ed59a2fba981bc5"
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 26
  total: 26
  not_honored: []
behavior_unverified_items: []
human_verification:

  - test: "Signed-in verified user opens /settings/tokens, creates a classic PAT, copies the one-time reveal, then revokes it from the list"
    expected: "List empty hero → Generate → classic form → reveal once → list shows prefix/note → revoke confirm removes token; password never accepted as git secret"
    why_human: "Vitest mocks apiClient; visual layout, clipboard, and real session cookie + RPC round-trip need a browser"
  - test: "Create a fine-grained PAT (selected repos + contents), then git ls-remote / push to a private repo over HTTPS using username+PAT"
    expected: "Token mints with octanest_fg_; Basic auth with PAT works; account password fails with PAT hint; private anon gets 401+WWW-Authenticate"
    why_human: "Integration tests cover API; Compose Traefik PathRegexp + real git client path needs operator smoke (scripts/smoke-git-https.sh skipped without Docker here)"
  - test: "On a repo page, open Clone HTTPS how-to; confirm aliases and CTA; check a long clone URL wraps/scrolls in the code block"
    expected: "How-to lists username aliases git/token/oauth2, password=PAT, Create CTA → /settings/tokens; long URL does not clip awkwardly"
    why_human: "08-12 backstop (overflow/wrap) is non-inferable from CSS presence alone; visual wrap needs human eyes"
  - test: "Review judgment prohibitions: no plaintext PAT at rest; PATs not usable as typed RPC Bearer"
    expected: "DB only stores token_hash; /api/rpc still session-only; docs say PATs are not RPC Bearer"
    why_human: "unverified-prohibition — human review recommended (LLM-judged from schema/grep/tests; judgment-tier must-NOT)"
---

# Phase 8: Git HTTPS & PATs Verification Report

**Phase Goal:** Users can authenticate git over HTTPS with personal access tokens (never account passwords) and manage those tokens in the UI

**Verified:** 2026-09-13T20:06:42Z  
**Status:** human_needed  
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | User can create, list, and revoke personal access tokens for HTTPS git (session RPC; PATs not RPC Bearer per D-01 / GIT-11 “where applicable”) | ✓ VERIFIED | `pat.createClassic` / `createFineGrained` / `list` / `revoke` in `crates/octanest-api/src/pat/mod.rs` + `rpc.rs`; tests `pat_create_classic_returns_one_time_token`, `pat_create_fine_grained_all_returns_fg_token`, `pat_list_omits_secret_token`, `pat_revoke_removes_from_list` PASS |
| 2 | User can clone, fetch, and push over HTTPS using a PAT; account password is rejected for git auth | ✓ VERIFIED | Smart HTTP `git_smart_http.rs` Basic → SHA-256 → `find_pat_by_token_hash`; `looks_like_pat` rejects non-prefix secrets; tests `git_smart_pat_push_fetch_happy_path`, `git_smart_basic_account_password_rejected_401` PASS |
| 3 | Token prefixes are `octanest_pat_` / `octanest_fg_` (D-08 locked) | ✓ VERIFIED | `octanest-core` `CLASSIC_PAT_PREFIX` / `FINE_GRAINED_PAT_PREFIX`; mint paths + api-client constants; unit asserts reject `ona_*` / github prefixes |
| 4 | HTTPS Smart HTTP URL is `/{owner}/{repo}.git` (D-18) | ✓ VERIFIED | Axum routes `/{owner}/{repo_git}/…` with `strip_git_suffix`; Traefik `PathRegexp(^/[^/]+/[^/]+\.git)` in `docker-compose.yml` |
| 5 | Private unauth git → 401 + `WWW-Authenticate`; PATs HTTPS-git-only (not RPC Bearer) (D-21 / D-01) | ✓ VERIFIED | `unauthorized_*` sets `WWW-Authenticate: Basic realm="Octanest Git"`; `git_smart_private_anon_401_www_authenticate` PASS; `find_pat_by_token_hash` only used from Smart HTTP; docs/API state session for RPC |
| 6 | Users manage tokens in UI (list/create classic+FG/revoke + CloneBox how-to) | ✓ VERIFIED | Routes `/settings/tokens`, `/new`, `/new/fine-grained`; `PatList` → `patListQueryOptions`; vitest `tokens.integration.test.ts` + `clone-box.pat.integration.test.ts` 16/16 PASS |
| 7 | Long clone URLs wrap or overflow-x-auto in how-to code blocks | ⚠️ insufficient_spec | CSS `overflow-x-auto … break-all` present in `pat-how-to.tsrx`; backstop truth — no held-out visual/DOM proof of wrap behavior |

**Score:** 6/7 truths verified (0 present, behavior-unverified; 1 abstained backstop)

**Locked decisions honored in code (not SUMMARY claims):**

- Prefixes: `octanest_pat_` / `octanest_fg_` (not `ona_*`)
- Clone path: `/{owner}/{repo}.git`
- Private unauth: 401 + WWW-Authenticate; PATs not RPC Bearer

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/octanest-db/migrations/*/0008_pats.sql` | PAT tables + `token_hash` | ✓ VERIFIED | Tri-dialect; hash UNIQUE; no plaintext column |
| `crates/octanest-db/src/pats.rs` | create/find/list/revoke | ✓ VERIFIED | Dialect helpers; wired via `Database` |
| `crates/octanest-core/src/pat_types.rs` | prefixes + DTOs | ✓ VERIFIED | Classic/FG kinds; list omits secret |
| `crates/octanest-api/src/pat/mod.rs` | PAT RPC | ✓ VERIFIED | createClassic/FG, list, revoke + verified gate |
| `crates/octanest-api/src/routes/git_smart_http.rs` | Smart HTTP auth | ✓ VERIFIED | Basic PAT, ACL, rate limit, cookie ignore |
| `crates/octanest-api/src/git/http_backend.rs` | CGI helper | ✓ VERIFIED | Exists + used by Smart HTTP |
| `apps/web/src/routes/settings/tokens*.tsrx` | Token UI | ✓ VERIFIED | List + classic + FG create |
| `apps/web/src/components/repo/pat-how-to.tsrx` | HTTPS how-to | ✓ VERIFIED | Wired into CloneBox + QuickSetup |
| `packages/api-client/src/index.ts` | Generated client | ✓ VERIFIED | `make rpc-sync-check` ok |
| `docker-compose.yml` + `scripts/smoke-git-https.sh` | Traefik + smoke | ✓ VERIFIED | PathRegexp + smoke target; smoke skipped (no Docker) this run |

### Key Link Verification

Automated `verify.key-links` failed on path-string greps (Rust crate modules do not path-import `.rs`/`.sql`). Manual wiring:

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `git_smart_http.rs` | DB `find_pat_by_token_hash` | Basic password → SHA-256 | ✓ WIRED | Not via `pat/mod.rs` path string — correct seam |
| `pat/mod.rs` | `octanest_db` pats | `create_pat` / `list_pats_for_user` / `revoke_pat` | ✓ WIRED | |
| `tokens.tsrx` / `pat-list.tsrx` | api-client | `patListQueryOptions` / `apiClient.pat.*` | ✓ WIRED | |
| `tokens.new*.tsrx` | api-client | `createClassic` / `createFineGrained` | ✓ WIRED | |
| `clone-box.tsrx` | `pat-how-to.tsrx` | `<PatHowTo />` | ✓ WIRED | Also QuickSetup |
| `app.rs` | Smart HTTP handlers | `/{owner}/{repo_git}/…` | ✓ WIRED | |
| `docker-compose.yml` | API Smart HTTP | Traefik PathRegexp | ✓ WIRED | Documented in CONFIGURATION |

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
| Create FG | `… pat_create_fine_grained_all_returns_fg_token` | ok | ✓ PASS |
| Push/fetch PAT | `… git_smart_pat_push_fetch_happy_path` | ok | ✓ PASS |
| Private 401 WWW-Auth | `… git_smart_private_anon_401_www_authenticate` | ok | ✓ PASS |
| Cookie ignored | `… git_smart_session_cookie_ignored_as_anon` | ok | ✓ PASS |
| Revoke | `… pat_revoke_removes_from_list` | ok | ✓ PASS |
| Schema 0008 | `… dialect_pats dialect_pats_migrate_0008_schema_presence` | ok | ✓ PASS |
| Tokens + how-to UI | `vitest run tokens.integration.test.ts clone-box.pat.integration.test.ts` | 16 passed | ✓ PASS |
| Client sync | `make rpc-sync-check` | ok | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| `scripts/smoke-git-https.sh` | `bash scripts/smoke-git-https.sh` | exit 0 — “docker not found… skipping” | ⚠️ SKIP (no Docker; not a failure of product code) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| **GIT-02** | 00–08, 12–13 | Clone/fetch/push HTTPS with PAT; not account password | ✓ SATISFIED | Smart HTTP + password-reject + push/fetch tests; UI how-to |
| **GIT-11** | 00–06, 08–11, 13 | Create/list/revoke PATs for HTTPS git (RPC/API where applicable) | ✓ SATISFIED | PAT RPC + UI; Phase 8 interprets “where applicable” as HTTPS-git-only (D-01) — management via session RPC, tokens not Bearer |

Orphaned requirements mapped to Phase 8: none (only GIT-02, GIT-11).

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts (26/26). Message: *All trackable CONTEXT.md decisions are honored by shipped artifacts.*

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `crates/octanest-api/tests/pat_rpc.rs` | GIT-11 | 10 | 0 | No | Behavioral / value | PASS |
| `crates/octanest-api/tests/git_smart_http.rs` | GIT-02 | 8 | 0 | No | Behavioral / status | PASS |
| `crates/octanest-db/tests/dialect_pats.rs` | GIT-11 | 2 | 0 | No | Value (schema) | PASS |
| `apps/web/.../tokens.integration.test.ts` | GIT-11 | (suite) | 0 | No | Behavioral (mocked RPC) | PASS — UI contract; backend covered separately |
| `apps/web/.../clone-box.pat.integration.test.ts` | GIT-02 | (suite) | 0 | No | Value (copy/CTA) | PASS |

**Disabled tests on requirements:** 0  
**Circular patterns detected:** 0  
**Insufficient assertions:** 0 blockers (web mocks are intentional; API nextest supplies behavioral proof)

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TBD/FIXME/XXX/TODO debt markers in key PAT/Smart HTTP/UI sources | — | — |

### Human Verification Required

### 1. Tokens UI create / reveal / revoke (browser)

**Test:** Signed-in verified user opens `/settings/tokens`, creates a classic PAT, copies the one-time reveal, then revokes it.  
**Expected:** Empty hero → Generate → classic form → one-time reveal → list shows prefix → revoke removes token.  
**Why human:** Vitest mocks `apiClient`; visual + real session RPC need a browser.

### 2. Fine-grained + real git HTTPS

**Test:** Mint FG PAT; `git ls-remote` / push to private repo with username+PAT; try account password; try private anon.  
**Expected:** `octanest_fg_` works; password fails with PAT hint; private anon 401+WWW-Authenticate; Traefik `.git` routes to API.  
**Why human:** This run’s smoke script skipped (no Docker).

### 3. How-to panel + long URL wrap (backstop)

**Test:** Open Clone HTTPS how-to; check aliases/CTA; force a long URL and confirm wrap/scroll.  
**Expected:** Aliases + PAT-as-password copy; CTA to `/settings/tokens`; no awkward clip.  
**Why human:** Backstop truth #7 abstained — CSS presence ≠ proven layout.

### 4. Judgment prohibitions (human ack)

**Test:** Confirm no plaintext PAT column; PAT cannot authenticate `/api/rpc` as Bearer.  
**Expected:** Hash-only storage; session cookie for RPC.  
**Why human:** unverified-prohibition — human review recommended.

### Gaps Summary

No blocking gaps on roadmap success criteria or GIT-02 / GIT-11. Phase goal is implemented and behaviorally proven at the API/integration layer. Overall status is `human_needed` for browser UAT, Compose smoke when Docker is available, backstop URL wrap, and judgment-tier prohibition acknowledgment — not because must-have truths failed.

### Inversion / disconfirmation notes

1. **GIT-11 “RPC/API where applicable”** — Intentional Phase 8 narrowing (D-01): PATs are HTTPS-git-only; token CRUD uses session RPC. Not a gap.  
2. **Web vitest mocks** — Do not alone prove RPC; nextest `pat_rpc` / `git_smart_http` close that gap.  
3. **Smoke skip** — Operator path unverified in this environment; product Traefik rule + Axum routes exist.

---

_Verified: 2026-09-13T20:06:42Z_  
_Verifier: Claude (gsd-verifier)_
