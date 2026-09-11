---
gsd_state_version: "1.0"
milestone: v1.0
current_phase: 06
current_phase_name: Self-Host Admin Bootstrap
status: executing
stopped_at: Completed 06-09-PLAN.md
last_updated: "2026-09-11T21:21:23.736Z"
last_activity: 2026-09-11
last_activity_desc: Phase 06 execution started
state_head: 325ea3b7dcad65aa6be8a7f38faa799eace4200c
progress:
  total_phases: 22
  completed_phases: 0
  total_plans: 41
  completed_plans: 39
milestone_name: milestone
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-09-09)

**Core value:** One forge you can trust in the cloud or on your own machines — without splitting into separate “hosted brand” vs “self-host software” products.
**Current focus:** Phase 06 — Self-Host Admin Bootstrap

## Current Position

Phase: 06 (Self-Host Admin Bootstrap) — EXECUTING
Plan: 9 of 10
Status: Ready to execute
Last activity: 2026-09-11 — Phase 06 execution started

Progress: Phases 1–5 complete; Phase 6 planned (06-00…06-09, docs as 06-07 wave 8)

## Performance Metrics

**Velocity:**

- Total plans completed: 31
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

### Pending Todos

- [ ] Fix signed-in home flicker on load (ui / minor) — `.planning/todos/pending/2026-09-11-fix-signed-in-home-flicker-on-load.md`

### Blockers/Concerns

None yet.

## Deferred Items

Items acknowledged and carried forward from previous milestone close:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-09-11T21:21:23.693Z
Stopped at: Completed 06-09-PLAN.md
Resume file: None
