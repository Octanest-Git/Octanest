---
phase: "06"
slug: "self-host-admin-bootstrap"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-11"
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo-nextest 0.9.x (API) + Vitest 5.x (web) |
| **Config file** | workspace Cargo; `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(bootstrap) \| test(seeded_admin) \| test(signup)'` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~60–180 seconds (full); ~15–40s quick |

---

## Sampling Rate

- **After every task commit:** Run quick API filter and/or `bun run --filter @octanest/web test:unit`
- **After every plan wave:** `cargo nextest run -p octanest-api` + web unit/integration
- **Before `$gsd-verify-work`:** Full suite (`make test`) must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-00-T1 | 00 | 0 | AUTH-06/07 | T-06-00 | Wave 0 API stubs (partial ENV, seed, allowlist, credentials) | integration | `cargo nextest run -p octanest-api -E 'test(bootstrap) \| test(seeded_admin) \| test(signup) \| test(forced_credentials) \| test(confirm_admin)'` | ❌ W0 | ⬜ pending |
| 06-00-T2 | 00 | 0 | AUTH-06/07 | — | Wave 0 web/dialect stubs incl. index SSR gate + `0006` columns | integration | `bun --cwd apps/web exec vitest run src/routes/setup.integration.test.ts src/routes/setup.credentials.integration.test.ts src/routes/dashboard.integration.test.ts src/routes/index.integration.test.ts src/components/chrome.integration.test.ts` | ❌ W0 | ⬜ pending |
| 06-01-T1 | 01 | 1 | AUTH-06/07 | T-06-01 | `0006_bootstrap_flags` + DB helpers | unit | `cargo test -p octanest-core --lib && cargo test -p octanest-db --lib migration_parity` | ❌ W0 | ⬜ pending |
| 06-01-T2 | 01 | 1 | AUTH-06/07 | T-06-02 | DTOs + dialect_auth column coverage | integration | `cargo test -p octanest-db --lib migration_parity && cargo test -p octanest-db --test dialect_auth && cargo test -p octanest-core --lib` | ⚠️ extend | ⬜ pending |
| 06-02-T1 | 02 | 2 | AUTH-06 | T-06 D-14 | Fail-closed boot checkpoint | human/gate | plan checkpoint (D-14) | — | ⬜ pending |
| 06-02-T2 | 02 | 2 | AUTH-06 | T-06-03 | ENV seed → must_change → confirm | integration | `cargo nextest run -p octanest-api -E 'test(seeded_admin) \| test(forced_credentials) \| test(confirm_admin)'` | ⚠️ extend | ⬜ pending |
| 06-02-T3 | 02 | 2 | AUTH-06 | — | Partial ENV + seed idempotency | integration | `cargo nextest run -p octanest-api -E 'test(bootstrap) \| test(seeded_admin) \| test(signup)'` | ⚠️ extend | ⬜ pending |
| 06-03-T1 | 03 | 3 | AUTH-07 | — | bootstrap_setup + allow_signup | integration | `cargo nextest run -p octanest-api -E 'test(bootstrap)'` | ⚠️ extend | ⬜ pending |
| 06-03-T2 | 03 | 3 | AUTH-07 | T-06 D-11 | Strict RPC allowlist + SSO reject | integration | `cargo nextest run -p octanest-api -E 'test(bootstrap) \| test(seeded_admin) \| test(signup)'` | ⚠️ extend | ⬜ pending |
| 06-04-T1 | 04 | 4 | AUTH-05/07 | — | provider_config.allow_signup + signup reject | integration | `cargo nextest run -p octanest-api -E 'test(signup) \| test(bootstrap) \| test(seeded_admin)'` | ⚠️ extend | ⬜ pending |
| 06-04-T2 | 04 | 4 | AUTH-05 | — | admin.auth allow_signup round-trip | integration | `cargo nextest run -p octanest-api -E 'test(admin_auth) \| test(signup)'` | ⚠️ extend | ⬜ pending |
| 06-05-T1 | 05 | 5 | AUTH-06/07 | T-06-11 | ssr-auth Cookie-forward + resolveAppAccessRedirect | build | `bun --cwd apps/web run build` | ❌ new | ⬜ pending |
| 06-05-T2 | 05 | 5 | AUTH-06/07 | T-06-15 | Shared root SSR gate (needs_setup + must_change) path matrix | unit | `bun --cwd apps/web exec vitest run src/lib/ssr-auth.gate.test.ts && bun --cwd apps/web run build` | ❌ new | ⬜ pending |
| 06-05-T3 | 05 | 5 | AUTH-06 | T-06 D-18/20 | `/` SSR tree selection (marketing vs SignedInHome) | integration | `bun --cwd apps/web exec vitest run src/routes/index.integration.test.ts && bun --cwd apps/web run build` | ❌ W0 | ⬜ pending |
| 06-06-T1 | 06 | 6 | AUTH-07 | — | Switch + /setup wizard | integration | `bun --cwd apps/web exec vitest run src/routes/setup.integration.test.ts && bun --cwd apps/web run build` | ❌ W0 | ⬜ pending |
| 06-06-T2 | 06 | 6 | AUTH-06 | T-06-14 | /setup/credentials forced change | integration | `bun --cwd apps/web exec vitest run src/routes/setup.credentials.integration.test.ts && bun --cwd apps/web run build` | ❌ W0 | ⬜ pending |
| 06-09-T1 | 09 | 6 | AUTH-06 | T-06-16 | `/dashboard` notFound | integration | `bun --cwd apps/web exec vitest run src/routes/dashboard.integration.test.ts && bun --cwd apps/web run build` | ❌ W0 | ⬜ pending |
| 06-09-T2 | 09 | 6 | AUTH-06/07 | — | `/signup` allow_signup false → notFound | build | `bun --cwd apps/web run build` | ⚠️ extend | ⬜ pending |
| 06-08-T1 | 08 | 7 | AUTH-05/07 | T-06-13 | Closed signup chrome/landing/login omit | integration | `bun --cwd apps/web exec vitest run src/components/chrome.integration.test.ts && bun --cwd apps/web run build` | ❌ W0 | ⬜ pending |
| 06-08-T2 | 08 | 7 | AUTH-05 | — | admin/auth Allow open signup Switch | build | `bun --cwd apps/web run build` | ⚠️ extend | ⬜ pending |
| 06-07-T1 | 07 | 8 | AUTH-06/07 | — | Docs + REQUIREMENTS reframe + COVERAGE | grep | `rg -n 'OCTANEST_ALLOW_SIGNUP\|empty.instance\|system-administrator' docs/CONFIGURATION.md .env.example .planning/REQUIREMENTS.md` | ⚠️ extend | ⬜ pending |
| 06-07-T2 | 07 | 8 | AUTH-06/07 | — | rpc-gen + phase smoke | integration | `cargo run -p octanest-api --bin rpc-gen && bun --cwd apps/web run build && cargo nextest run -p octanest-api -E 'test(bootstrap) \| test(seeded_admin) \| test(signup)'` | ⚠️ extend | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Extend `crates/octanest-api/tests/auth_bootstrap.rs` — partial ENV, allow_signup on setup, strict RPC allowlist
- [ ] Extend seed tests — username `system-administrator`, `must_change_credentials`, `OCTANEST_ALLOW_SIGNUP`
- [ ] New API tests — forced credential change RPC; signup blocked when `allow_signup=false`
- [ ] Web: `/setup` Switch + `/setup/credentials` form integration tests
- [ ] Web: signup/chrome omit + dashboard notFound tests
- [ ] Web: `apps/web/src/routes/index.integration.test.ts` — D-18/D-20 home SSR tree gate (needs_setup vs SignedInHome vs marketing)
- [ ] Migration `0006_bootstrap_flags` dialect triple + `dialect_auth` coverage for new columns

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First-boot wizard UX on empty compose | AUTH-07 | Full browser + empty DB | Bring up empty self-host without ADMIN env; complete wizard; confirm admin session |
| ENV seed then forced credentials UX | AUTH-06 | Browser + seeded admin | Set both ADMIN env; boot; login; confirm redirect to `/setup/credentials` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
