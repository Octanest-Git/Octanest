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

- **After every task commit:** Targeted `cargo test -p octanest-api --test …` / `bun run test:unit` for the touched area
- **After every plan wave:** Workspace Rust tests + web unit (+ `make db-matrix` when migrations/token CRUD touched)
- **Before `$gsd-verify-work`:** Full suite green + human UAT per UI-SPEC (banner, verify/reset copy, anti-enumeration)
- **Max feedback latency:** 60 seconds (quick path)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-W0-* | 00/01 | 0 | AUTH-04/12 | — | Wave 0 stubs for verify/reset/gate | scaffold | (created in Wave 0 tasks) | ❌ W0 | ⬜ pending |
| TBD | TBD | 1 | AUTH-04 | T-05-* | Unverified → `auth.email_unverified` on privileged ping | integration | `cargo test -p octanest-api --test auth_verify_gate` | ❌ W0 | ⬜ pending |
| TBD | TBD | 1 | AUTH-04 | T-05-* | Local signup unverified; OTP/token consume sets verified | integration | `cargo test -p octanest-api --test auth_verify_reset` | ❌ W0 | ⬜ pending |
| TBD | TBD | 1 | AUTH-04 | T-05-* | IdP `email_verified=true` sets verified | unit/integration | workos/oidc + DB assert | ❌ W0 | ⬜ pending |
| TBD | TBD | 1 | AUTH-04 | T-05-* | Admin seed auto-verified | integration | seed path test | ❌ W0 | ⬜ pending |
| TBD | TBD | 1 | AUTH-05 | — | Signup without invite still succeeds | integration | extend `auth_signup` | ✅ extend | ⬜ pending |
| TBD | TBD | 2 | AUTH-12 | T-05-* | Reset request always same ok; mail only local-password | integration | recording EmailSender | ❌ W0 | ⬜ pending |
| TBD | TBD | 2 | AUTH-12 | T-05-* | Redeem sets password, revokes others, sets cookie | integration | multi-session pattern | ❌ W0 | ⬜ pending |
| TBD | TBD | 2 | AUTH-12 | T-05-* | SSO-only no send / redeem error class | integration | password_hash null user | ❌ W0 | ⬜ pending |
| TBD | TBD | 1 | PLAT-08 | — | Token migrate + CRUD on 3 dialects | integration | extend `dialect_auth` | ❌ W0 | ⬜ pending |
| TBD | TBD | 3 | UI | — | OTP wrapper + verify/reset pages | component | vitest unit/integration | ❌ W0 | ⬜ pending |

*Planner replaces TBD Task IDs / Plan / Wave / Threat Ref when PLAN.md files are written.*

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-api/tests/auth_verify_reset.rs` — issue/consume/rate-limit/anti-enumeration
- [ ] `crates/octanest-api/tests/auth_verify_gate.rs` — `require_verified` + env-gated privileged ping
- [ ] Extend `crates/octanest-db/tests/dialect_auth.rs` (or sibling) for `0003` tokens + `set_email_verified`
- [ ] Web: InputOtp + verify/reset route smoke tests
- [ ] rpc-gen / api-client regeneration after `email_verified` DTO change

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Verify banner + resend UX | AUTH-04 | Chrome placement / theme | Sign in unverified; confirm banner under header; resend cooldown copy |
| `/verify` + `/reset-password` chrome | AUTH-04/12 | Browser + OTP a11y | Light/dark; 8-slot OTP; magic-link + code paths |
| Anti-enumeration success panel | AUTH-12 | Timing/UX judgment | Request reset for unknown email; same panel as known local account |
| Live SMTP/Resend E2E | AUTH-12 | Requires operator secrets | Only when keys configured; default CI uses log-sink |
| Phase success criteria UAT | ALL | Human checkpoint | Roadmap success criteria 1–3 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
