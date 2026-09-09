---
status: testing
phase: 02-multi-db-storage
source:
  - 02-01-SUMMARY.md
  - 02-02-SUMMARY.md
  - 02-03-SUMMARY.md
  - 02-04-SUMMARY.md
  - 02-05-SUMMARY.md
started: 2026-09-09T15:40:47Z
updated: 2026-09-09T16:21:10Z
---

## Current Test

number: 4
name: Dialect mismatch fails fast
expected: |
  With `DATABASE_URL=postgres://…` and `OCTANEST_DB_DIALECT=mysql`, starting the API exits immediately
  with a clear "does not match DATABASE_URL scheme" message (no hanging server). Password is not printed
  if the URL contains one.
awaiting: user response

## Tests

### 1. Cold Start Smoke Test (Postgres Compose)
expected: From a clean state, `make smoke` brings the stack up; `/`, `/health`, `system.health`, and `system.db_probe` (dialect=postgres, increasing probe_count) succeed; smoke tears down cleanly.
result: pass
verified_by: agent (`make smoke`; WSL docker socket issues — used PATH wrapper to docker.exe)

### 2. MySQL overlay smoke
expected: `make smoke-mysql` succeeds; `system.db_probe` returns `"dialect":"mysql"` and probe_count increases.
result: pass
verified_by: agent (`make smoke-mysql`; compose-smoke translates COMPOSE_PROFILES into --profile)

### 3. SQLite overlay smoke
expected: `make smoke-sqlite` succeeds with no database container; file appears under `./var/`; `system.db_probe` returns `"dialect":"sqlite"` and probe_count increases.
result: pass
verified_by: agent (`make smoke-sqlite`; WSL+docker.exe cannot bind Linux paths — sqlite-host-dir.sh uses Windows temp bind + mirrors DB to ./var)

### 4. Dialect mismatch fails fast
expected: With `DATABASE_URL=postgres://…` and `OCTANEST_DB_DIALECT=mysql`, starting the API exits immediately with a clear "does not match DATABASE_URL scheme" message (no hanging server). Password is not printed if the URL contains one.
result: pending

### 5. SQLite migrate + probe without Compose
expected: Point `DATABASE_URL=sqlite:./target/tmp/uat/octanest.db`, run migrate and dialect probe. Write/read succeeds; parent dirs are created automatically.
result: pending

### 6. Empty-target dialect switch
expected: `make db-switch-dialect` / migrate `--assert-empty` succeeds on an empty SQLite target, then refuses the same target once populated (unless `--force-empty`).
result: pending

### 7. Operator docs agree on three dialects
expected: `docs/database.md`, README (`make up-sqlite`), `.env.example` (`sqlite:./var/octanest.db`), and `make help` all describe the same three dialect paths; no stale `sqlite:./data/` docs.
result: pending

### 8. CI db-matrix is defined
expected: `.github/workflows/ci.yml` contains a `db-matrix` job covering postgres, mysql, and sqlite with `dialect_probe`, and the compose job validates the sqlite overlay config (no live compose bring-up in CI).
result: pending

## Summary

total: 8
passed: 3
issues: 0
pending: 5
skipped: 0
blocked: 0

## Gaps

[none yet]
