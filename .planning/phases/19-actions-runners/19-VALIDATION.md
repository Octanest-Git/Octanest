---
phase: "19"
slug: "actions-runners"
status: planned
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-16"
updated: "2026-09-16"
---

# Phase 19 — Validation Strategy

> Per-phase validation contract. Seeded from `19-RESEARCH.md` Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: cargo-nextest; Web: Vitest via Bun |
| **Config file** | `.config/nextest.toml`; `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(actions_)|test(runner_)|test(commit_status)'` |
| **Full suite command** | `make test` (+ `make rpc-sync-check`; `make smoke-actions` skip-ok without Docker) |
| **Estimated runtime** | ~90–300 seconds (quick); longer with smoke |

---

## Sampling Rate

- **After every task commit:** targeted nextest / Vitest filter for touched area
- **After every plan wave:** actions nextest + web Actions Vitest + `make rpc-sync-check` after RPC changes
- **Before `/gsd-verify-work`:** `make test` + `make smoke-actions` (skip-ok) green
- **Max feedback latency:** 300 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|----------------|-----------------|-----------|-------------------|-------------|--------|
| 19-00-T* | 00 | 0 | ACT-01..07 | T-19-SC | Wave 0 RED Rust stubs | integration | nextest list actions_* | ⬜ | ⬜ |
| 19-01-T* | 01 | 0 | ACT-03..05 | T-19-SC | Wave 0 web + smoke stubs | component/smoke | Vitest actions + smoke-actions.sh | ⬜ | ⬜ |
| 19-02-T* | 02 | 1 | ACT-03,07 | T-19-01 | Schema + LOG_DIR + factory reset | integration | dialect_actions | ⬜ | ⬜ |
| 19-03-T* | 03 | 2 | ACT-01 | T-19-02 | Workflow parse + run/job rows | integration | actions_workflow_parse | ⬜ | ⬜ |
| 19-04-T* | 04 | 3 | ACT-01,02,06,07 | T-19-03 | Tracer push→queue→FetchTask | integration | actions_triggers + runner protocol | ⬜ | ⬜ |
| 19-05-T* | 05 | 4 | ACT-04,06 | T-19-04 | Full runner protocol + labels | integration | actions_runner_protocol | ⬜ | ⬜ |
| 19-06-T* | 06 | 4 | ACT-02 | T-19-05 | pull_request dispatch contract | integration | actions_triggers | ⬜ | ⬜ |
| 19-07-T* | 07 | 5 | ACT-03 + Ph13 | T-19-06 | Commit statuses publish/query | integration | commit_statuses | ⬜ | ⬜ |
| 19-08-T* | 08 | 5 | ACT-04,05 | T-19-07 | Official image + Compose | smoke/docs | smoke-actions + Dockerfile rg | ⬜ | ⬜ |
| 19-09-T* | 09 | 6 | ACT-03 | T-19-08 | Actions UI list/detail/logs | component | Vitest actions routes | ⬜ | ⬜ |
| 19-10-T* | 10 | 6 | ACT-06,07 | T-19-09 | Tokens, secrets, repo enable | integration + UI | actions_secrets + admin | ⬜ | ⬜ |
| 19-11-T* | 11 | 7 | ACT-01..07 | T-19-01 | Docs + smoke + phase gate | mixed | smoke-actions + docs rg | ⬜ | ⬜ |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-api/tests/actions_workflow_parse.rs` — ACT-01
- [ ] `crates/octanest-api/tests/actions_triggers.rs` — ACT-02
- [ ] `crates/octanest-api/tests/actions_rpc.rs` — ACT-03
- [ ] `crates/octanest-api/tests/actions_runner_protocol.rs` — ACT-06
- [ ] `crates/octanest-api/tests/actions_dispatch_policy.rs` — ACT-07
- [ ] `crates/octanest-api/tests/commit_statuses.rs` — Phase 13 surface / D-ACT-15
- [ ] `crates/octanest-db/tests/dialect_actions.rs` — migration parity
- [ ] `apps/web/src/routes/$owner.$repo.actions.integration.test.ts` — ACT-03 UI
- [ ] `apps/web/src/routes/$owner.$repo.actions.$run.integration.test.ts` — ACT-03 detail
- [ ] `scripts/smoke-actions.sh` + `make smoke-actions` — ACT-04/05

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Official runner registers and runs a real job | ACT-04 | Needs Docker + runner image | Compose profile actions; push workflow; watch job green |
| Standalone runner against public origin | ACT-05 | Multi-host networking | Register with non-loopback ORIGIN; checkout works in job container |
| Phase 13 required check blocks merge | ORG-05/PR-08 | Phase 13 not this phase | Verify status contexts exist and are queryable for Phase 13 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 300s
- [ ] `nyquist_compliant: true` — owned by `/gsd-validate-phase`

**Approval:** pending execute + validate-phase
