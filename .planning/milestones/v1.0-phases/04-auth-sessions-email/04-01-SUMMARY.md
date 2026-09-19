---
phase: 04-auth-sessions-email
plan: "01"
subsystem: auth
tags: [sqlx, migrations, sessions, users, argon2-ready, multi-dialect]

requires:
  - phase: 02-multi-db-storage
    provides: "octanest-db DbPool, per-dialect migrator, migration_parity"
provides:
  - "Auth DTOs (ProviderMode, UserPublic, AuthSettingsPublic) in octanest-core"
  - "0002_auth migrations on postgres/mysql/sqlite (users, sessions, auth_identities, instance_auth_settings)"
  - "Dialect-branched CRUD for users/sessions/identities/settings"
  - "dialect_auth integration test scaffold"
affects:
  - 04-02-email-adapters
  - 04-03-passwords-sessions
  - 04-04-local-auth-rpc
  - 04-05-oidc-workos
  - 04-06-profile-admin

tech-stack:
  added: []
  patterns:
    - "DbPool match-only dialect branching in octanest-db CRUD modules"
    - "Hash-only columns for password_hash and sessions.token_hash"
    - "App-normalized lowercase email (no CITEXT) for dialect parity"

key-files:
  created:
    - crates/octanest-core/src/auth_types.rs
    - crates/octanest-db/migrations/postgres/0002_auth.sql
    - crates/octanest-db/migrations/mysql/0002_auth.sql
    - crates/octanest-db/migrations/sqlite/0002_auth.sql
    - crates/octanest-db/src/users.rs
    - crates/octanest-db/src/sessions.rs
    - crates/octanest-db/src/auth_identities.rs
    - crates/octanest-db/src/auth_settings.rs
    - crates/octanest-db/tests/dialect_auth.rs
  modified:
    - crates/octanest-core/src/lib.rs
    - crates/octanest-db/src/lib.rs

key-decisions:
  - "Session create takes explicit session id + RFC3339 expires_at strings for multi-dialect binds"
  - "var/ already gitignored — no .gitignore change needed for avatar volume"
  - "Macro-based row mappers keep concrete sqlx Row types without dialect leaks"

patterns-established:
  - "Auth persistence modules mirror probe.rs DbPool match + Database thin wrappers with database not configured"
  - "dialect_* tests skip when DATABASE_URL unset"

requirements-completed: [AUTH-01, AUTH-02, AUTH-08]

duration: 7min
completed: 2026-09-09
---

# Phase 4 Plan 01: Auth Schema, DTOs & DB CRUD Summary

**Multi-dialect `0002_auth` schema plus octanest-core auth DTOs and octanest-db CRUD for users, sessions, identities, and instance settings**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-09T23:10:16Z
- **Completed:** 2026-09-09T23:17:32Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Shared serde DTOs (`ProviderMode`, `UserPublic`, signup/login/profile requests, `AuthSettingsPublic`) with GitHub-like username validation and reserved list
- `0002_auth.sql` on all three dialects with UNIQUE email/username, nullable `password_hash`, `sessions.token_hash`, FK cascade, and seeded `instance_auth_settings`
- `Database` helpers for user/session/identity/settings CRUD; `dialect_auth` migrate + round-trip test (skip-or-pass without `DATABASE_URL`)

## Task Commits

Each task was committed atomically:

1. **Task 1: Auth DTOs in octanest-core** - `383a93c` (feat)
2. **Task 2: 0002_auth migrations on all three dialects** - `d63c6bd` (feat)
3. **Task 3: DB CRUD modules + dialect_auth test scaffold** - `c628482` (feat)

**Plan metadata:** pending (this commit) `docs(04-01): complete auth schema and DB CRUD plan`

_Note: requirements AUTH-01/02/08 are foundation-only here (persistence primitives); full signup/login/profile UX lands in later Phase 4 plans._

## Files Created/Modified

- `crates/octanest-core/src/auth_types.rs` — Auth DTOs + `validate_username` / `is_reserved_username`
- `crates/octanest-core/src/lib.rs` — `pub mod auth_types` + re-exports
- `crates/octanest-db/migrations/*/0002_auth.sql` — users, sessions, auth_identities, instance_auth_settings
- `crates/octanest-db/src/users.rs` — insert/find/update_profile/count
- `crates/octanest-db/src/sessions.rs` — create/find/touch/delete/delete_all_for_user
- `crates/octanest-db/src/auth_identities.rs` — upsert/find by provider subject
- `crates/octanest-db/src/auth_settings.rs` — get/update singleton settings
- `crates/octanest-db/src/lib.rs` — modules + `Database` auth methods
- `crates/octanest-db/tests/dialect_auth.rs` — `migrate_auth_and_user_round_trip`

## Decisions Made

- Session `create` accepts an explicit `id` (caller-owned UUID) in addition to the plan sketch — required for PK without pulling `uuid` into the db crate yet
- Timestamp parameters are RFC3339 `&str` for portable binds across PG/MySQL/SQLite
- Skipped `.gitignore` edit: existing `var/` already covers `var/uploads/`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Session create requires `id` parameter**
- **Found during:** Task 3 (sessions CRUD)
- **Issue:** Plan interface omitted session PK; table requires TEXT/CHAR PK
- **Fix:** `sessions::create` and `Database::create_session` take `id: &str`
- **Files modified:** `crates/octanest-db/src/sessions.rs`, `crates/octanest-db/src/lib.rs`
- **Verification:** dialect_auth round-trip on SQLite
- **Committed in:** `c628482` (Task 3)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Necessary for correct schema usage; no scope creep.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Persistence foundation ready for 04-02 (email) and 04-03 (Argon2id + cookie sessions)
- No dialect branching leaked outside `octanest-db`

## Self-Check: PASSED

- Created files exist on disk
- Commits `383a93c`, `d63c6bd`, `c628482` present in git log
- `cargo test -p octanest-core --lib` green
- `cargo test -p octanest-db --lib migration_parity` green
- `cargo test -p octanest-db --test dialect_auth` green (skip/pass)
- No `sqlx::query!` in new auth modules

---
*Phase: 04-auth-sessions-email*
*Completed: 2026-09-09*
