---
phase: 1
slug: monorepo-scaffold
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-09-09
approved: 2026-09-09
validated: 2026-09-09
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + vitest (JS) |
| **Config file** | `packages/api-client/vitest.config.ts`; Cargo workspace |
| **Quick run command** | `cargo test -p octanest-api && bun run --filter @octanest/api-client test` |
| **Full suite command** | `cargo test --workspace && bun run --filter @octanest/api-client test && make rpc-sync-check && docker compose config && bunx turbo run build --filter=@octanest/web` |
| **CI** | `.github/workflows/ci.yml` — rust / js / rpc-sync / compose |
| **Estimated runtime** | Quick ~60s · Full ~5–8 min (Compose smoke when Docker Engine up) |

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
| 01-00-01 | 01 | 0 | PLAT-* | — | N/A | infra | scaffold configs exist | ✅ | ✅ green |
| 01-02-01 | 02 | 2 | PLAT-05, PLAT-06 | T-01-03 | Reject missing/wrong `Octanest-RPC-Version` | integration | `cargo test -p octanest-api --test rpc_http` | ✅ `crates/octanest-api/tests/rpc_http.rs` | ✅ green |
| 01-02-02 | 02 | 2 | PLAT-06 | T-01-04 | CORS prod requires allowlist | unit | `cargo test -p octanest-api cors_` | ✅ `crates/octanest-api/src/cors.rs` | ✅ green |
| 01-02-03 | 02 | 2 | PLAT-06 | T-01-05 | echo HTTP+WS (+ 8KiB cap) | integration | `cargo test -p octanest-api --test rpc_http --test rpc_ws` | ✅ `tests/rpc_http.rs`, `tests/rpc_ws.rs` | ✅ green |
| 01-02-04 | 02 | 2 | PLAT-06 | T-01-06 | codegen sync | script | `make rpc-sync-check` | ✅ `scripts/check-rpc-sync.sh` | ✅ green |
| 01-03-01 | 03 | 3 | PLAT-04, PLAT-10, PLAT-11 | T-01-07 | web build | build | `bunx turbo run build --filter=@octanest/web` | ✅ `apps/web` | ✅ green |
| 01-04-01 | 04 | 4 | PLAT-01 | T-01-09/10 | Compose config + smoke | smoke | `docker compose config` / `./scripts/compose-smoke.sh` | ✅ | ✅ green (smoke verified 2026-09-09) |
| 01-05-01 | 05 | 5 | PLAT-* | T-01-12/13 | CI gates | ci | `.github/workflows/ci.yml` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky / env-blocked*

**Requirement coverage:** PLAT-01, PLAT-04, PLAT-05, PLAT-06, PLAT-10, PLAT-11 → COVERED (automated and/or UAT).

---

## Wave 0 Requirements

- [x] `crates/octanest-api/tests/rpc_http.rs` — health (`/health`), echo, version reject
- [x] `crates/octanest-api/tests/rpc_ws.rs` — health/echo over WS
- [x] `packages/api-client` vitest config + smoke test
- [x] `scripts/check-rpc-sync.sh` — regen + diff
- [x] Vitest + Cargo workspace test harness installed

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions | UAT |
|----------|-------------|------------|-------------------|-----|
| Landing first-viewport brand/CTA composition | UI-SPEC / PLAT-04 | Visual judgment | Open `/` — mark + Octanest hero + Get started / Explore Octanest | ✅ passed (`01-UAT.md`) |
| Theme system → light/dark force persists | UI-SPEC | Browser preference UX | Toggle theme; refresh; confirm persistence | ✅ passed |
| Footer Status navigation | CONTEXT D-26 | Simple UX | Click footer Status → `/status` live health | ✅ passed (agent + human) |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s quick
- [x] `nyquist_compliant: true` set in frontmatter after plans pass Dimension 8

**Approval:** 2026-09-09

---

## Validation Audit 2026-09-09

| Metric | Count |
|--------|-------|
| Gaps found | 0 (MISSING) |
| Partial refreshed | 1 (`01-04-01` smoke → green) |
| Escalated | 0 |
| Suite re-run | cargo + vitest + rpc-sync + compose config + web build — green |
