---
phase: 19-actions-runners
plan: "10"
subsystem: api
tags: [actions, secrets, runners, admin, octane]

requires:
  - phase: 19-actions-runners
    provides: runner protocol + Actions UI list/detail
provides:
  - AES-GCM Actions secrets encrypt/store + FetchTask injection
  - Admin registration token + listRunners RPC/UI
  - Repo Actions enable toggle + secrets CRUD UI
  - Registered-runners-only policy assertion (ACT-07)
affects: [19-11]

actuals:
  tokens: 20129
  tasks: 3
  commits: 3

tech-stack:
  added: [aes-gcm, getrandom]
  patterns: [OCTANEST_ACTIONS_SECRETS_KEY derive via SHA-256, write-only secrets]

key-files:
  created:
    - crates/octanest-api/src/actions/secrets.rs
    - crates/octanest-api/tests/actions_secrets.rs
    - apps/web/src/routes/admin/runners.tsrx
    - apps/web/src/components/repo/actions-settings-panel.tsrx
    - apps/web/src/routes/$owner.$repo.settings.actions.tsrx
  modified:
    - crates/octanest-api/src/actions/rpc.rs
    - crates/octanest-api/src/actions/runner_proto.rs
    - crates/octanest-core/src/action_types.rs
    - crates/octanest-db/src/actions.rs
    - packages/api-client/src/index.ts
    - docs/CONFIGURATION.md
    - crates/octanest-api/tests/actions_dispatch_policy.rs

key-decisions:
  - "Secrets encrypted AES-GCM; list returns name/updated_at only"
  - "Admin createRegistrationToken shows plaintext once; env bootstrap still works"
  - "No managed-minutes path — policy test + docs reinforce ACT-07"

patterns-established:
  - "decrypted_secrets_for_repo for FetchTask injection"
  - "Admin runners nav entry + Actions settings panel"

requirements-completed: [ACT-06, ACT-07]

coverage:
  - id: D1
    description: actions_secrets + dispatch_policy nextest
    requirement: ACT-06
    verification:
      - kind: unit
        ref: cargo nextest -E 'test(actions_secrets)|test(actions_dispatch_policy)'
        status: pass
    human_judgment: false

plan_head_before: 8fc33decaf7ea33b06a27a4eceec9947d3e3ee26
duration: 45min
completed: 2026-09-16
status: complete
---

# Phase 19 Plan 10: Admin tokens & secrets Summary

**AES-GCM repo Actions secrets, Admin registration-token UX, and registered-runners-only policy (ACT-06/07).**

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Deduplicated concurrent DB helper inserts**
- **Found during:** Task 1
- **Issue:** Parallel WIP left duplicate `delete_secret_by_name` / `list_secret_ciphertexts` / `list_runners` and lib.rs wrappers.
- **Fix:** Kept CipherRow + bool-delete variants; removed duplicates.
- **Files modified:** `crates/octanest-db/src/actions.rs`, `crates/octanest-db/src/lib.rs`
- **Commit:** 50bc8ed

**2. [Rule 1 - Bug] Aligned RPC DTOs after type-name drift**
- **Found during:** Task 1
- **Issue:** Concurrent edits renamed secret DTOs (`ActionSecretPublic` alias) and expanded `ActionRunnerPublic`.
- **Fix:** Mapped RPC handlers to the stabilized core types; rpc-gen regenerated client.
- **Commit:** 50bc8ed

## Self-Check: PASSED

- FOUND: `crates/octanest-api/src/actions/secrets.rs`
- FOUND: `apps/web/src/routes/admin/runners.tsrx`
- FOUND: `50bc8ed`, `8cd68c2`, `9911298`
