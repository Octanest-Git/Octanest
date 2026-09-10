---
phase: "5"
slug: "cloud-verify-reset"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-10"
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `05-RESEARCH.md` § Validation Architecture.
> Per-Task Verification Map filled by planner (05-01…05-07; revision 1 split tracer).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: `cargo test` (workspace); TS: Vitest in `apps/web` + `@octanest/api-client` |
| **Config file** | crates’ `[[test]]` / apps/web vitest projects |
| **Quick run command** | `cargo test -p octanest-api --test auth_verify_reset --test auth_verify_gate` |
| **Full suite command** | `make test` + `make db-matrix` (when DB touched) + `bun run test` in apps/web + api-client vitest if client changed |
| **Estimated runtime** | ~30–60s quick / ~3–8m full + dialect matrix |

---

## Sampling Rate

- **After every task commit:** Targeted `cargo test -p octanest-api --test …` / `bun --cwd apps/web run build` for the touched area
- **After every plan wave:** Workspace Rust tests + web build (+ dialect_auth when migrations/token CRUD touched)
- **Before `$gsd-verify-work`:** Full suite green + human UAT per UI-SPEC (banner, verify/reset copy, anti-enumeration)
- **Max feedback latency:** 60 seconds (quick path)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-T1 | 01 | 1 | AUTH-04 | T-05-03 | 0003 tokens + CRUD + UserPublic.email_verified field | unit + parity | `cargo test -p octanest-core --lib && cargo test -p octanest-db --lib migration_parity` | ❌→T1 | ⬜ pending |
| 05-01-T2 | 01 | 1 | AUTH-04 / PLAT-08 | — | Token migrate + CRUD + verified helpers on dialects | integration | `cargo test -p octanest-db --lib migration_parity && cargo test -p octanest-db --test dialect_auth` | ✅ extend | ⬜ pending |
| 05-02-T1 | 02 | 2 | AUTH-04 | T-05-01…T-05-04 | Unverified → `auth.email_unverified`+403; OTP verify → me.email_verified + ping ok | integration | `cargo test -p octanest-api --test auth_verify_gate --test auth_verify_reset` | ❌ W0→T1 | ⬜ pending |
| 05-02-T2 | 02 | 2 | AUTH-04 | T-05-04 | privileged_ping unknown outside allowlist | integration | `cargo test -p octanest-api --test auth_verify_gate` | ❌→T2 | ⬜ pending |
| 05-03-T1 | 03 | 3 | AUTH-04 | T-05-05…T-05-08 | Issue/resend/rate-limit/magic+OTP; PUBLIC_ORIGIN links | integration | `cargo test -p octanest-api --test auth_verify_reset` | ❌→T1 | ⬜ pending |
| 05-03-T2 | 03 | 3 | AUTH-04, AUTH-05 | T-05-08 | Signup auto-send verify; admin seed verified; open signup | integration | `cargo test -p octanest-api --test auth_signup --test auth_verify_reset` | ✅ extend | ⬜ pending |
| 05-03-T3 | 03 | 3 | AUTH-04 | — | Reserved usernames `verify`, `reset-password` | unit | `cargo test -p octanest-core --lib` | ✅ | ⬜ pending |
| 05-04-T1 | 04 | 4 | AUTH-12 | T-05-09, T-05-12 | Reset request anti-enumeration; mail only local-password | integration | `cargo test -p octanest-api --test auth_verify_reset` | ❌→T1 | ⬜ pending |
| 05-04-T2 | 04 | 4 | AUTH-12 | T-05-10, T-05-11 | Redeem sets password, revokes others, Set-Cookie | integration | `cargo test -p octanest-api --test auth_verify_reset` | ❌→T2 | ⬜ pending |
| 05-05-T1 | 05 | 5 | AUTH-04 | T-05-13 | IdP email_verified=true sets verified_at | unit/integration | `cargo test -p octanest-api --lib auth::external auth::workos auth::oidc && cargo test -p octanest-api --test auth_verify_gate` | ✅ extend | ⬜ pending |
| 05-05-T2 | 05 | 5 | AUTH-04 | T-05-14 | clear_email_verification helper required + unit test | unit | `grep -q 'fn clear_email_verification' …/verify_reset.rs && cargo test -p octanest-api --lib clear_email_verification` | ❌→T2 | ⬜ pending |
| 05-06-T1 | 06 | 6 | — | T-05-SC | Human confirm input-otp@1.5.0 before install | checkpoint | blocking-human | n/a | ⬜ pending |
| 05-06-T2 | 06 | 6 | AUTH-04 | T-05-SC, T-05-15, T-05-16 | rpc-gen + InputOtp + /verify build | build | `cargo run -p octanest-api --bin rpc-gen && bun --cwd apps/web run build` | ❌→T2 | ⬜ pending |
| 05-06-T3 | 06 | 6 | AUTH-04 | — | VerifyBanner under header for unverified | build + human-check | `bun --cwd apps/web run build` | ❌→T3 | ⬜ pending |
| 05-07-T1 | 07 | 7 | AUTH-12, AUTH-05 | T-05-17, T-05-18, T-05-20 | /reset-password + forgot link; no invite UI | build + human-check | `bun --cwd apps/web run build` | ❌→T1 | ⬜ pending |
| 05-07-T2 | 07 | 7 | AUTH-04 | T-05-19 | Disabled New repository CTA; phase regression | build + integration | `bun --cwd apps/web run build && cargo test -p octanest-api --test auth_verify_gate --test auth_verify_reset && cargo test -p octanest-api --test auth_signup` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Planned as part of 05-02-T1: `crates/octanest-api/tests/auth_verify_reset.rs`
- [x] Planned as part of 05-02-T1: `crates/octanest-api/tests/auth_verify_gate.rs`
- [x] Planned as part of 05-01-T2: extend `crates/octanest-db/tests/dialect_auth.rs` for `0003` tokens + verified helpers
- [x] Web OTP/verify coverage via build + human-check in 05-06/05-07
- [x] rpc-gen / api-client regeneration in 05-06-T2 after DTO/RPC surface complete

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Verify banner + resend UX | AUTH-04 | Chrome placement / theme | Sign in unverified; confirm banner under header; resend cooldown copy |
| `/verify` + `/reset-password` chrome | AUTH-04/12 | Browser + OTP a11y | Light/dark; 8-slot OTP; magic-link + code paths |
| Anti-enumeration success panel | AUTH-12 | Timing/UX judgment | Request reset for unknown email; same panel as known local account |
| Live SMTP/Resend E2E | AUTH-12 | Requires operator secrets | Only when keys configured; default CI uses log-sink |
| Phase success criteria UAT | ALL | Human checkpoint | Roadmap success criteria 1–3 |
| UI backstops (overflow/long-text) | UI | Held-out visual | max-w-md wrap; banner wrap on narrow — insufficient_spec → human_needed if no evidence |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
