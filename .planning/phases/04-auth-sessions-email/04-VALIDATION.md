---
phase: 4
slug: auth-sessions-email
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-09-10
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: `cargo test` (workspace); TS: Vitest `^5` in `@octanest/api-client` |
| **Config file** | crates’ `[[test]]` / `packages/api-client/vitest.config.ts` |
| **Quick run command** | `cargo test -p octanest-api --lib && cargo test -p octanest-db --lib` |
| **Full suite command** | `make test` + `bun run --filter @octanest/api-client test` + `make db-matrix` |
| **Estimated runtime** | ~30s quick / ~3–8m full + dialect matrix |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p octanest-api --lib` + targeted `--test` for the touched area
- **After every plan wave:** Run `make test` + `make db-matrix` (when DB touched) + api-client vitest if client changed
- **Before `/gsd-verify-work`:** Full suite must be green + Compose smoke signup/login on default Postgres
- **Max feedback latency:** 30 seconds (quick path)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-W0-* | 00 | 0 | AUTH-* | — | N/A | infra | Wave 0 stubs below | ❌ W0 | ⬜ pending |
| 04-*-* | TBD | TBD | AUTH-01 | T-4-pass | Argon2id PHC; no password logs | integration | `cargo test -p octanest-api --test auth_signup` | ❌ W0 | ⬜ pending |
| 04-*-* | TBD | TBD | AUTH-02 | T-4-sess | Opaque HttpOnly cookie; new session on login | integration | `cargo test -p octanest-api --test auth_session` | ❌ W0 | ⬜ pending |
| 04-*-* | TBD | TBD | AUTH-03 | T-4-sess | Logout deletes session row; logout-all revokes all | integration | `cargo test -p octanest-api --test auth_session` | ❌ W0 | ⬜ pending |
| 04-*-* | TBD | TBD | AUTH-08 | T-4-avatar | Server-chosen path; size/type allowlist | integration | `cargo test -p octanest-api --test profile_avatar` | ❌ W0 | ⬜ pending |
| 04-*-* | TBD | TBD | AUTH-09 | — | LogSink only; no network | unit | `cargo test -p octanest-api email::log_sink` | ❌ W0 | ⬜ pending |
| 04-*-* | TBD | TBD | AUTH-10 | T-4-email | Typed addresses; no header injection | unit | `cargo test -p octanest-api email::smtp` | ❌ W0 | ⬜ pending |
| 04-*-* | TBD | TBD | AUTH-11 | T-4-email | Bearer + User-Agent; mock HTTP | unit | `cargo test -p octanest-api email::resend` | ❌ W0 | ⬜ pending |

*Planner must replace TBD Task IDs when PLAN.md files are written. Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-api/tests/auth_signup.rs` — stubs for AUTH-01 + welcome LogSink
- [ ] `crates/octanest-api/tests/auth_session.rs` — stubs for AUTH-02/03 cookie jar via tower oneshot
- [ ] `crates/octanest-api/tests/profile_avatar.rs` — stubs for AUTH-08
- [ ] `crates/octanest-api` email unit tests — AUTH-09/10/11 (smtp/resend with mock)
- [ ] `crates/octanest-db/migrations/*/0002_auth.sql` (+ migration parity)
- [ ] `crates/octanest-db/tests/dialect_auth.rs` — signup/session on each dialect (or extend dialect_probe)
- [ ] Shared cookie-aware RPC test helpers in api tests
- [ ] Optional: `wiremock` / `httpmock` dev-dep for Resend/WorkOS HTTP

*Existing `rpc_http` / `rpc_db_probe` tests are the template for new http oneshot tests.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `/login` and `/signup` render with Octanest chrome | AUTH-01/02 UI | Browser chrome / theme | Open routes in light and dark; confirm brand shell |
| Post-login redirect to `/dashboard` or `returnTo` | AUTH-02 | Navigation UX | Sign in from landing vs deep link |
| Live WorkOS / Resend / SMTP E2E | AUTH-10/11 + providers | Requires operator secrets | Only when keys configured; default CI uses mocks/log-sink |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter after planner fills Task IDs

**Approval:** pending
