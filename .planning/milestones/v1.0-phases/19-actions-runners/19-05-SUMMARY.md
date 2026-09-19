---
phase: 19-actions-runners
plan: "05"
subsystem: api
tags: [actions, runner-protocol, prost, registration-tokens]

requires:
  - phase: 19-actions-runners
    provides: Push→FetchTask tracer + Actions schema
provides:
  - Pinned proto/runner.proto + prost codegen via build.rs
  - Full Register/Declare/FetchTask/UpdateTask/UpdateLog handlers
  - Registration tokens + OCTANEST_RUNNER_REGISTRATION_TOKEN bootstrap
affects: [19-06, 19-07, 19-09]

actuals:
  tokens: 5200
  tasks: 2
  commits: 2

tech-stack:
  added: [prost 0.13, prost-build 0.13]
  patterns: [HTTP+JSON handlers aligned to prost types, SHA-256 token hashing, label[:schema[:args]] match]

key-files:
  created:
    - crates/octanest-api/proto/runner.proto
    - crates/octanest-api/build.rs
    - crates/octanest-api/src/actions/tokens.rs
  modified:
    - crates/octanest-api/Cargo.toml
    - crates/octanest-api/src/actions/runner_proto.rs
    - crates/octanest-api/src/actions/mod.rs
    - crates/octanest-api/tests/actions_runner_protocol.rs
    - crates/octanest-db/src/actions.rs
    - crates/octanest-db/src/lib.rs

key-decisions:
  - "Keep JSON HTTP mount; include prost-generated types for pinned proto compatibility"
  - "protoc via user-local ~/.cache/protoc-* fallback in build.rs (no sudo)"

patterns-established:
  - "Runner auth is bearer/body token only; cookies ignored (D-ACT-18)"

requirements-completed: [ACT-04, ACT-06, ACT-07]

coverage:
  - id: D1
    description: Register + env bootstrap + Declare/Fetch/labels
    requirement: ACT-06
    verification:
      - kind: integration
        ref: cargo nextest run -p octanest-api -E 'test(actions_runner_protocol)'
        status: pass
    human_judgment: false

plan_head_before: 8647ec58554989f40af7f8dbeccf657560276b61
duration: 25min
completed: 2026-09-16
status: complete
---

# Phase 19 Plan 05: Runner protocol + prost Summary

**Pinned `runner.proto` with prost codegen; Register/Declare/FetchTask/UpdateTask/UpdateLog green including label match, registration tokens, and env bootstrap — cookies still rejected.**

## What Shipped

- Vendored `crates/octanest-api/proto/runner.proto` + `build.rs` (PROTOC cache fallback).
- `tokens` module: mint/consume registration tokens; `OCTANEST_RUNNER_REGISTRATION_TOKEN`.
- Expanded `runner_proto` handlers; DB helpers for claim-by-labels, declare labels, job status.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] protoc missing (no sudo)**
- **Found during:** Task 1
- **Issue:** `prost-build` requires `protoc`; apt install needs password.
- **Fix:** Downloaded official protoc 29.3 to `~/.cache/protoc-29.3` and taught `build.rs` to fall back there.
- **Files modified:** `crates/octanest-api/build.rs`
- **Commit:** fb50e62

Task 0 (prost legitimacy) auto-advanced — only [VERIFIED] prost crates added.

## Self-Check: PASSED
