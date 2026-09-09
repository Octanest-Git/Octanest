---
phase: 1
slug: monorepo-scaffold
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-09-09
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + vitest 5.x (JS) |
| **Config file** | none yet — Wave 0 / Plan 01–02 installs (`vitest.config.ts`, Cargo workspace) |
| **Quick run command** | `cargo test -p octanest-api --lib && bunx vitest run packages/api-client` |
| **Full suite command** | `cargo test --workspace && bunx vitest run && make rpc-gen && git diff --exit-code -- packages/api-client` |
| **Estimated runtime** | Quick ~60s · Full ~5–8 min (incl. Compose smoke when scheduled) |

---

## Sampling Rate

- **After every task commit:** Run quick command (or package-scoped subset)
- **After every plan wave:** Run full suite for touched layers + Compose smoke when Compose plan lands
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds for quick; 600 seconds for full

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-00-01 | 01 | 0 | PLAT-* | — | N/A | infra | scaffold configs exist | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 2 | PLAT-05, PLAT-06 | T-01-01 | Reject missing/wrong `Octanest-RPC-Version` | integration | `cargo test -p octanest-api rpc_` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 2 | PLAT-06 | T-01-02 | CORS prod requires allowlist (unit/config test) | unit | `cargo test -p octanest-api cors_` | ❌ W0 | ⬜ pending |
| 01-02-03 | 02 | 2 | PLAT-06 | — | echo HTTP+WS | integration | `cargo test -p octanest-api` | ❌ W0 | ⬜ pending |
| 01-02-04 | 02 | 2 | PLAT-06 | — | codegen sync | script | `make rpc-gen && git diff --exit-code -- packages/api-client` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 3 | PLAT-04, PLAT-10, PLAT-11 | — | web build | build | `bunx turbo run build --filter=web` | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 4 | PLAT-01 | T-01-03 | Compose up healthy | smoke | `docker compose up --wait` + curl | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Planner must expand this table to cover every PLAN task with `<automated>` verify.*

---

## Wave 0 Requirements

- [ ] `crates/octanest-api/tests/rpc_http.rs` — stubs for health, echo, version reject
- [ ] `crates/octanest-api/tests/rpc_ws.rs` — stubs for health, echo over WS
- [ ] `packages/api-client` vitest config + smoke test stub
- [ ] `scripts/check-rpc-sync.sh` — regen + diff
- [ ] Vitest + Cargo workspace test harness installed

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Landing first-viewport brand/CTA composition | UI-SPEC / PLAT-04 | Visual judgment | Open `/` — mark + Octanest hero + Get started / Explore Octanest; no dashboard clutter |
| Theme system → light/dark force persists | UI-SPEC | Browser preference UX | Toggle theme; refresh; confirm persistence |
| Footer Status navigation | CONTEXT D-26 | Simple UX | Click footer Status → `/status` shows live health |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s quick
- [ ] `nyquist_compliant: true` set in frontmatter after plans pass Dimension 8

**Approval:** pending
