---
phase: 2
slug: multi-db-storage
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-09-09
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + existing compose smoke (bash) |
| **Config file** | `Cargo.toml` workspace / `.github/workflows/ci.yml` |
| **Quick run command** | `cargo test -p octanest-db --lib` |
| **Full suite command** | `cargo test --workspace` (CI: `db-matrix` + existing rust/js/rpc-sync/compose jobs) |
| **Estimated runtime** | ~30–120s local unit; matrix legs longer with services |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p octanest-db --lib`
- **After every plan wave:** Run `cargo test --workspace` (or package-scoped tests for that wave)
- **Before `/gsd-verify-work`:** Full suite must be green including CI matrix expectations
- **Max feedback latency:** 120 seconds for quick unit path

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | 01+ | 1+ | PLAT-07 | T-02-* | Dialect mismatch fails closed | unit | `cargo test -p octanest-db resolve_dialect` | ❌ W0 | ⬜ pending |
| TBD | 01+ | 1+ | PLAT-08 | T-02-* | Migrations + probe per dialect | integration | CI `db-matrix` | ❌ W0 | ⬜ pending |
| TBD | 02+ | 2+ | PLAT-08 | — | Compose smoke probe RPC | smoke | `make smoke` / smoke-mysql / smoke-sqlite | ❌ W0 | ⬜ pending |

*Planner must replace TBD rows with concrete task IDs when PLAN.md files are written. Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-db` dialect resolve unit tests (schemes + mismatch)
- [ ] Migration filename parity check across `migrations/{postgres,mysql,sqlite}/`
- [ ] `crates/octanest-db` integration probe tests gated on `DATABASE_URL` (skip if unset)
- [ ] CI `db-matrix` job for postgres/mysql/sqlite
- [ ] Extend compose smoke for `system.db_probe`; Make targets for mysql/sqlite overlays

*Existing Phase 1 cargo/CI infrastructure covers runners; Wave 0 adds dialect-specific stubs/jobs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live three-profile Compose bring-up on a workstation | PLAT-08 / ROADMAP #3 | CI compose job is config-validation oriented (research Layer 4) | `make up` / `make up-mysql` / `make up-sqlite` then corresponding `make smoke*` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s for unit path
- [ ] `nyquist_compliant: true` set in frontmatter after plans lock task map

**Approval:** pending
