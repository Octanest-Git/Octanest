---
gsd_state_version: "1.0"
milestone: v1.0
current_phase: 07
current_phase_name: Git Repos & Browse
status: executing
stopped_at: Completed 07-20-PLAN.md
last_updated: "2026-09-12T19:53:21.495Z"
last_activity: 2026-09-12
last_activity_desc: Phase 07 execution started
state_head: fcb8e14ad040094fd918018dcce674cea114fdee
progress:
  total_phases: 22
  completed_phases: 0
  total_plans: 63
  completed_plans: 62
milestone_name: milestone
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-09-09)

**Core value:** One forge you can trust in the cloud or on your own machines — without splitting into separate “hosted brand” vs “self-host software” products.
**Current focus:** Phase 07 — Git Repos & Browse

## Current Position

Phase: 07 (Git Repos & Browse) — EXECUTING
Plan: 21 of 22 (next: 07-20 CR-01 archive argv)
Status: Ready to execute
Last activity: 2026-09-12 — Completed 07-19 CR-02 gap closure

Progress: Phase 7 plans 00–19 complete (incl. CR-02); gap-closure remaining: 07-20, 07-21

See also: `phases/07-git-repos-browse/07-*-PLAN.md` · `07-COVERAGE.md` · `07-CONTEXT.md` · `07-UI-SPEC.md` · `07-RESEARCH.md`

## Performance Metrics

**Velocity:**

- Total plans completed: 41
- Average duration: —
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-monorepo-scaffold | 5 | 5 | — |
| 02-multi-db-storage | 5 | 5 | — |
| 03-brand-shell-theme | 6 | 6 | — |
| 4 | 8 | - | - |
| 5 | 7 | - | - |
| 06 | 10 | - | - |

**Recent Trend:**

- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
| Phase 04-auth-sessions-email P01 | 7min | 3 tasks | 11 files |
| Phase 04-auth-sessions-email P02 | 3min | 2 tasks | 8 files |
| Phase 04-auth-sessions-email P03 | 3min | 2 tasks | 6 files |
| Phase 04-auth-sessions-email P04 | 4min | 2 tasks | 12 files |
| Phase 04-auth-sessions-email P05 | 7min | 2 tasks | 12 files |
| Phase 04-auth-sessions-email P06 | 10min | 2 tasks | 17 files |
| Phase 04-auth-sessions-email P07 | 4min | 2 tasks | 14 files |
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 04-auth-sessions-email P08 | 5min | 3 tasks | 5 files |
| Phase 05 P01 | 6 min | 2 tasks | 8 files |
| Phase 05-cloud-verify-reset P02 | 3 min | 2 tasks | 8 files |
| Phase 05-cloud-verify-reset P03 | 9 min | 3 tasks | 15 files |
| Phase 05 P04 | 6 min | 2 tasks | 6 files |
| Phase 05 P05 | 5 min | 2 tasks | 5 files |
| Phase 05 P06 | 4min | 3 tasks | 10 files |
| Phase 05 P07 | 4 min | 2 tasks | 4 files |
| Phase 06-self-host-admin-bootstrap P00 | 8min | 2 tasks | 10 files |
| Phase 06-self-host-admin-bootstrap P01 | 6min | 2 tasks | 10 files |
| Phase 06 P02 | 4min | 3 tasks | 7 files |
| Phase 06-self-host-admin-bootstrap P03 | 3min | 2 tasks | 5 files |
| Phase 06-self-host-admin-bootstrap P04 | 5min | 2 tasks | 5 files |
| Phase 06 P05 | 8min | 3 tasks | 9 files |
| Phase 06-self-host-admin-bootstrap P06 | 8min | 2 tasks | 10 files |
| Phase 06-self-host-admin-bootstrap P09 | 7min | 2 tasks | 6 files |
| Phase 06-self-host-admin-bootstrap P08 | 13min | 2 tasks | 8 files |
| Phase 06 P07 | 2min | 2 tasks | 8 files |
| Phase 07-git-repos-browse P00 | 3 min | 1 tasks | 10 files |
| Phase 07-git-repos-browse P16 | 2 min | 1 tasks | 2 files |
| Phase 07 P01 | 1min | 3 tasks | 3 files |
| Phase 07-git-repos-browse P02 | 6min | 2 tasks | 12 files |
| Phase 07-git-repos-browse P12 | 4min | 1 tasks | 18 files |
| Phase 07-git-repos-browse P17 | 1min | 1 tasks | 5 files |
| Phase 07 P13 | 5min | 1 tasks | 5 files |
| Phase 07-git-repos-browse P03 | 10min | 3 tasks | 101 files |
| Phase 07 P04 | 11min | 2 tasks | 27 files |
| Phase 07-git-repos-browse P14 | 4min | 1 tasks | 8 files |
| Phase 07 P05 | 7min | 1 tasks | 15 files |
| Phase 07-git-repos-browse P15 | 10min | 1 tasks | 15 files |
| Phase 07 P06 | 10min | 2 tasks | 16 files |
| Phase 07-git-repos-browse P07 | 4min | 1 tasks | 9 files |
| Phase 07 P08 | 7min | 2 tasks | 10 files |
| Phase 07-git-repos-browse P18 | 5min | 1 tasks | 6 files |
| Phase 07 P09 | 6min | 2 tasks | 13 files |
| Phase 07-git-repos-browse P10 | 10min | 3 tasks | 20 files |
| Phase 07-git-repos-browse P11 | 3min | 2 tasks | 3 files |
| Phase 07-git-repos-browse P19 | 2 min | 2 tasks | 4 files |
| Phase 07 P20 | 2 min | 2 tasks | 5 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Roadmap: fine granularity — 22 thin phases
- UI: `@octanejs/tanstack-start` + ShadCN + Base UI + Tailwind CSS v4 (CSS config)
- Theme: system default; user can force light or dark
- Stack: Rust backend, RPC codegen, multi-DB, gitoxide-preferred
- Phase 3: squircle mark, ShadCN semantic tokens, Vite PWA assets-only SW; mobile one-row header + burger menu
- [Phase 04]: Session create takes explicit id + RFC3339 expires_at for multi-dialect binds — Sessions table requires PK without adding uuid to octanest-db yet; string timestamps bind portably
- [Phase 04]: var/ already covers avatar upload volume path — Existing gitignore var/ makes var/uploads/ redundant
- [Phase 04]: lettre default-features off + rustls (aws-lc-rs); reqwest 0.13 uses `rustls` feature — Current crate versions differ from plan's rustls-tls naming
- [Phase 04]: ResendSender::with_base_url for wiremock; production URL https://api.resend.com/emails — Testable without live Resend
- [Phase 04]: argon2 0.6 SaltString + hash_password_with_salt for PHC strings — Current argon2 crate API differs from RESEARCH 0.5-style OsRng example
- [Phase 04]: Hex 32-byte session tokens; SHA-256 hex at rest; SessionService owns env Secure flag — Matches discretion locks; avoids Domain attribute / Vite proxy pitfall
- [Phase 04]: auth.unauthenticated → HTTP 401; other auth errors → 400 — Prefer consistent JSON err with distinct unauthenticated status for clients
- [Phase 04]: Welcome email failures logged only; signup still succeeds — Mail adapter outages must not block account creation (D-20)
- [Phase 04]: Admin seed only when OCTANEST_ADMIN_* set and count_users==0 — T-04-13; Phase 6 owns interactive wizard
- [Phase 04]: WorkOS AuthKit PKCE + authenticate_with_code mints Octanest session (not sealed cookies) — D-07 / T-04-17
- [Phase 04]: OIDC issuer SSRF: https-only; reject localhost/10/8/link-local/metadata — T-04-16 ASVS L1
- [Phase 04]: Avatar public URL path /uploads/avatars/{user_id}.webp stored in users.avatar_path — UI consumes avatar_url directly; filesystem path stays under uploads_dir
- [Phase 04]: AppState email is Arc<RwLock> for hot-rebuild on admin.auth.update_settings — D-09 email_provider changes must take effect without restart
- [Phase 04]: AuthSettingsPublic returns workos_client_id display + ENV configured booleans — T-04-22 secrets never leave ENV; admin UI needs non-secret display fields
- [Phase 04]: Hand-authored Base UI shadcn wrappers rather than CLI scaffold — Matches Phase 3 pattern so contracts stay stable for wave plans
- [Phase 04]: Post-auth redirects via window.location.assign(safeReturnTo) — Supports untyped returnTo paths before profile/admin routes exist
- [Phase 04]: Vite proxies /api/auth for WorkOS/OIDC start URLs — Local SSO CTAs must reach the API
- [Phase 4]: Vite proxies /api/user and /uploads so avatar POST/preview work in local Vite dev
- [Phase 4]: Human UAT checkpoint approved with no defects — no post-UAT code changes
- [Phase 05]: Upsert re-fetches by (user_id, purpose) after ON CONFLICT for stable row return
- [Phase 05]: UserPublic.email_verified field added without wiring user_to_public (deferred to 05-02)
- [Phase 05]: privileged_ping allowlist is {development,dev,test,compose} per Open Q2 RESOLVED
- [Phase 05]: issue_verify is a library helper for tests; full resend/email RPC deferred to 05-03
- [Phase 05]: issue_count via 0004 migration for soft hourly rate limits — UNIQUE replace-on-resend cannot count issues from created_at alone
- [Phase 05]: Session-scoped verify redeem for attempt capping — Wrong OTP via find_by_otp_hash cannot increment attempts
- [Phase 05]: Swallow rate_limit into ok on password reset request (D-28 anti-enumeration) — Surfacing auth.rate_limited only for known emails would enumerate accounts
- [Phase 05]: auth.sso_only on reset redeem only; request never reveals SSO-only — D-26/D-28 + UI-SPEC SSO copy
- [Phase 05]: Re-apply IdP-trust on existing SSO identity link path when provider asserts verified email — Returning users with newly verified IdP email should not stay stuck unverified
- [Phase 05]: OIDC trusts email_verified only when claim is Some(true); false/absent uses local verify — D-03/D-15 IdP-trust must not treat missing claim as verified
- [Phase 5]: Human approved input-otp@1.5.0 legitimacy gate before install (05-06-T1)
- [Phase 5]: Token auto-consume invalid_token while signed-in maps to wrong-user copy on /verify
- [Phase 05]: After anti-enumeration success, Enter reset code advances to redeem without magic link
- [Phase 05]: Dashboard New repository always disabled in Phase 5; hint differs by email_verified (D-12)
- [Phase 06]: Wave 0 is RED-only — no GREEN/REFACTOR; later 06-xx plans turn stubs green
- [Phase 06]: support::lock_admin_env owns the ENV mutex (does not import untracked bootstrap.rs)
- [Phase 06]: setup.credentials stub documents UI-SPEC without static-importing the missing route module
- [Phase 06]: BootstrapSetupRequest/UpdateAuthSettingsRequest allow_signup serde-default false (fail closed)
- [Phase 06]: provider_config exposes allow_signup; DB error fails closed to false
- [Phase 06]: No instance_flags table — columns on instance_auth_settings + users
- [Phase 06]: D-14 fail_closed: keep exit(1) when both ADMIN ENV set and maybe_seed_admin returns Err (do not serve wizard fallback)
- [Phase 06]: D-14 fail_closed confirmed: exit(1) on ENV seed Err; no serve_wizard_fallback
- [Phase 06]: Landed untracked bootstrap.rs as tracked module with auth.confirm_admin_credentials
- [Phase 06]: allow_signup false persistence proven by pre-opening settings then wizard close
- [Phase 06]: SSO reject_if_setup_required already present — no auth_callbacks change in 06-03
- [Phase 06]: Signup/provider_config enforcement already landed in 06-03 — 06-04 locks RPC contracts + rpc_gen DTOs
- [Phase 06]: provider_config tested post-bootstrap (needs_setup allowlist blocks it on empty instance)
- [Phase 06]: Land route edits on .tsrx (in-flight Octane rename) instead of restoring deleted .tsx
- [Phase 06]: UserPublic.must_change_credentials added to api-client + rpc_gen for SSR gate
- [Phase 06]: Client redirectIfNeedsSetup demoted to PE after shared root SSR gate landed
- [Phase 06]: Land Switch/setup/credentials/login on .tsrx (Octane rename in flight)
- [Phase 06]: Split /setup into layout Outlet so /setup/credentials nests without blank child
- [Phase 06]: confirmAdminCredentials + BootstrapSetupRequest.allow_signup added to api-client with UI
- [Phase 06]: Land dashboard/signup on .tsrx (Octane rename in flight) — plan .tsx paths reconciled
- [Phase 06]: throw notFound() (not soft redirect / AuthShell) for /dashboard and closed /signup
- [Phase 06]: Signup API-unreachable: do not 404; API allow_signup remains authority
- [Phase 06]: [Phase 06]: Land chrome/admin edits on .tsrx (Octane rename in flight)
- [Phase 06]: [Phase 06]: needs_setup omits account CTAs; Sign up fail-closed until allow_signup===true
- [Phase 06]: AUTH-05 keeps checkbox; v1 note clarifies allow_signup supersedes always-open cloud signup
- [Phase 06]: Document cloud OCTANEST_ALLOW_SIGNUP=true in manifests — no Compose file change (Open Q2)
- [Post-06]: Client server-state via `@octanejs/tanstack-query` (not Zustand) — shared `auth.me` / bootstrap / providerConfig / admin settings; forms stay local `useState`
- [Post-06]: Author UI in Octane `.tsrx` with `@{` / `@if` / `@else` / `@for`; do not mix React-style `return (` components with Rivet directives (breaks Vite import-protection HMR)
- [Post-06]: OIDC reqwest connect/request timeouts; mock-oauth2-server healthcheck + compose issuer pointing at reachable host
- [Post-06]: Setup wizard can choose auth stack (local/WorkOS/OIDC public fields); sys-admin factory reset wipes instance back to needs_setup
- [Phase 07]: Git backend is system `git` CLI 2.5+ (fail boot if missing); gitoxide deferred until feature-complete; keep GitBackend abstraction (amends GIT-09)
- [Phase 07]: GitHub-like browse IA (`/{owner}/{repo}`, tree/blob/raw/blame/compare); private=owner-only until Phase 10; create via `/new` with full templates
- [Phase 07]: Wave 0 is RED-only — no CliGitBackend or repo RPC handlers; later 07-xx plans turn stubs green
- [Phase 07]: octanest-git exports parse_git_version/assert_git_version placeholders that Err until implementation
- [Phase 07]: Wave 0 web stubs RED-only for /new wall + home CTA → /new — Production /new and CTA wiring deferred to 07-13 / 07-04; Nyquist discoverability first
- [Phase 07]: new.integration.test uses @vite-ignore dynamic import while route absent — Static import('./new') fails Vite transform with 0 tests; runtime import keeps suite discoverable and RED
- [Phase 07]: D-14: owner_repo_path — public URLs /{owner}/{repo} with reserved-name denylist
- [Phase 07]: D-33: fail_boot_git — refuse API boot if git missing or < 2.5
- [Phase 07]: D-32: CLI-primary GitBackend; promote GitBackend noun; CliGitBackend now, GixGitBackend later
- [Phase 07]: 07-02: proceed_locked for D-14 owner_repo_path + D-33 fail_boot_git
- [Phase 07]: 07-02: MySQL soft-delete uniqueness via generated active_name column
- [Phase 07]: Duplicate create returns stable repo.name_taken for inline /new UI (D-12)
- [Phase 07]: CreateRepoRequest.visibility optional — omit uses instance default_visibility else public (D-08)
- [Phase 07]: Fail-boot git gate + Dockerfile/Compose left to 07-17 (D-33)
- [Phase 07]: Fail boot with eprintln + exit(1) when git missing or < 2.5.0 (D-33)
- [Phase 07]: Install distro git in API image; bind ./var/repos without overriding default OCTANEST_REPOS_DIR
- [Phase 07]: Added $owner.$repo Outlet layout so Quick setup index registers like /setup
- [Phase 07]: Template/license/gitignore remain None-only placeholders until 07-03
- [Phase 07]: Public/Private via button group until radio-group in 07-09
- [Phase 07]: License picker uses native select for full SPDX performance; stack/gitignore keep UI Select
- [Phase 07]: Unknown SPDX IDs seed SPDX-License-Identifier stub LICENSE when text not vendored
- [Phase 07]: repo.createDefaults RPC supplies default_visibility + stack/gitignore catalogs
- [Phase 07]: listMine requires session only (not email verified)
- [Phase 07]: default_branch via optional UpdateProfileRequest + dedicated Save default branch UI
- [Phase 07]: D-18: remark-gfm → rehype-sanitize last for README XSS safety
- [Phase 07]: D-19: in-repo TextMate grammars for tsrx/ripple (not TS/JS alias)
- [Phase 07]: Shiki cool-biased themes: github-light + github-dark singleton
- [Phase 07]: Private ACL stub is owner-only until Phase 10; identical repo.not_found (D-23–D-25)
- [Phase 07]: Blob soft limit 1 MiB for RPC preview and raw truncation headers (D-20)
- [Phase 07]: Soft RepoNotFound for repo.not_found (identical D-25 copy); Clone/Download stub until 07-08
- [Phase 07]: History RPC names: repo.commits/commit/compare/blame with soft patch/blame caps
- [Phase 07]: Compare empty UI copy: Nothing to compare for these refs.
- [Phase 07]: RPC names repo.branchCreate/Rename/Delete; soft-protect error repo.default_branch_protected
- [Phase 07]: Non-owner branch mutate returns repo.not_found; Branches/Tags UI deferred to 07-18
- [Phase 07]: Archive bytes collected under 120s timeout (no new streaming crates; T-07-SC)
- [Phase 07]: Empty/unborn ref archive → 404 repo.archive_unavailable structured JSON
- [Phase 07]: Clone box archives disabled when repo empty; HTTPS copy always available
- [Phase 07]: Hand-authored Dialog/AlertDialog from official Base UI (no third-party registry)
- [Phase 07]: Tags UI list+download only; tip SHA shown without per-ref updated timestamps
- [Phase 07]: RPC camelCase repo.updateVisibility / repo.softDelete; soft-delete confirmName server-side; disk purge deferred to 07-10
- [Phase 07]: Settings visibility UI reuses /new Public/Private buttons (no new radio-group)
- [Phase 07]: Cleanup frequency/retention via ENV (not new DB columns) to avoid schema migration in Phase 7
- [Phase 07]: orphan_reconcile keeps soft-deleted dirs until retention; ENV knobs for interval/gc
- [Phase 07]: ARCHITECTURE documents CliGitBackend as shipped adapter and GixGitBackend as future-only (D-32)
- [Phase 07]: nyquist_compliant left false until /gsd-validate-phase; Wave 0 checklist marked complete conceptually
- [Phase 07]: rpc-gen produced no api-client diff — client already matched repo.* dispatch
- [Phase 07]: reject_option_like_branch treats any leading- hyphen as repo.invalid_ref (covers -d/-D/-m/-M/-f and case variants) — Matches validate_treeish; no separate token allowlist needed for CR-02
- [Phase 07]: Known flags -m/-D stay before --; user from/to/name always after end-of-options — Defense in depth so operands cannot slide into option position (CR-02)
- [Phase 07]: RED for archive CR-01 asserts HTTP-boundary invalid ref message so intentional fail remains after 07-19 CLI validate_treeish — Bare 4xx+no-file would unexpected-GREEN after plan 19
- [Phase 07]: validate_ref allows slashy hierarchical refs and rejects leading hyphen like archive — WR-02 / GIT-05 browse parity with CR-01 defense-in-depth

### Pending Todos

- *(none)* — signed-in home flicker todo completed (moved to `.planning/todos/completed/`)

### Blockers/Concerns

None. Phase 7 planning should assume Query session cache + Octane `.tsrx` + decisions in `07-CONTEXT.md` (including GIT-09 amendment).

## Deferred Items

Items acknowledged and carried forward from previous milestone close:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

Post-06 shipped polish (not deferred — already in tree): see `phases/06-self-host-admin-bootstrap/deferred-items.md` § Post-close addendum.

## Session Continuity

Last session: 2026-09-12T19:53:21.438Z
Stopped at: Completed 07-20-PLAN.md
Resume file: None
