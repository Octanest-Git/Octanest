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
| 06-W0-* | 00 | 0 | AUTH-06/07 | — | Wave 0 stubs for gates | integration | see Wave 0 | ❌ W0 | ⬜ pending |
| 06-*-* | TBD | TBD | AUTH-06 | T-06 ENV seed | Seed + fail-closed | integration | `cargo nextest run -p octanest-api -E 'test(seeded_admin)'` | ⚠️ extend | ⬜ pending |
| 06-*-* | TBD | TBD | AUTH-07 | T-06 wizard | Wizard + allow_signup | integration | `cargo nextest run -p octanest-api -E 'test(bootstrap)'` | ⚠️ extend | ⬜ pending |

*Planner fills concrete Task IDs when PLAN.md files are written. Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Extend `crates/octanest-api/tests/auth_bootstrap.rs` — partial ENV, allow_signup on setup, strict RPC allowlist
- [ ] Extend seed tests — username `system-administrator`, `must_change_credentials`, `OCTANEST_ALLOW_SIGNUP`
- [ ] New API tests — forced credential change RPC; signup blocked when `allow_signup=false`
- [ ] Web: `/setup` Switch + `/setup/credentials` form integration tests
- [ ] Web: signup/chrome omit + dashboard notFound tests
- [ ] Migration `0006` dialect triple + `dialect_auth` coverage for new columns

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
