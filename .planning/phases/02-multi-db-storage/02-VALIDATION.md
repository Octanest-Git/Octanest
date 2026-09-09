---
phase: 2
slug: multi-db-storage
status: planned
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
| 02-01-T1 | 02-01 | 1 | PLAT-07 | T-02-01 | Dialect mismatch fails closed; credentials redacted from errors | unit | `cargo test -p octanest-db --lib resolve_dialect` and `cargo test -p octanest-db --lib redact_url_hides_password` | ❌ W0 | ⬜ pending |
| 02-01-T2 | 02-01 | 1 | PLAT-07 | T-02-05 | One pool type per dialect; SQLite WAL + foreign_keys + bounded pool | unit | `cargo build -p octanest-db && cargo test -p octanest-api --tests` | ❌ W0 | ⬜ pending |
| 02-02-T1 | 02-02 | 2 | PLAT-08 | T-02-02 | Migration sets cannot drift between dialects | unit | `cargo test -p octanest-db --lib migration_parity` | ❌ W0 | ⬜ pending |
| 02-02-T2 | 02-02 | 2 | PLAT-08 | T-02-03 | Probe SQL uses bound placeholders only | unit+build | `cargo test --workspace` | ❌ W0 | ⬜ pending |
| 02-02-T3 | 02-02 | 2 | PLAT-08 | T-02-01, T-02-13 | Migrate+probe round-trip; SQLite parent dir auto-create; migrate CLI refuses a non-empty target | integration+cli | `DATABASE_URL=sqlite:./target/tmp/verify/octanest.db cargo test -p octanest-db --test dialect_probe`, `cargo test -p octanest-db --test sqlite_paths`, and `DATABASE_URL=sqlite:./target/tmp/verify-cli/octanest.db cargo run -q -p octanest-db --bin migrate -- --assert-empty` | ❌ W0 | ⬜ pending |
| 02-03-T1 | 02-03 | 3 | PLAT-07 | T-02-08 | Bad dialect config or dead DB aborts startup; auto-migrate gated | integration | `DATABASE_URL=postgres://u:p@localhost/x OCTANEST_DB_DIALECT=mysql cargo run -q -p octanest-api --bin octanest-api` exits 1 | ❌ W0 | ⬜ pending |
| 02-03-T2 | 02-03 | 3 | PLAT-08 | T-02-06, T-02-09 | Probe errors return a stable code without internals; version header still enforced | integration | `cargo test -p octanest-api --test rpc_db_probe` | ❌ W0 | ⬜ pending |
| 02-03-T3 | 02-03 | 3 | PLAT-08 | T-02-10 | Generated client cannot drift from Rust types | contract | `make rpc-sync-check && bun run --filter @octanest/api-client test` | ❌ W0 | ⬜ pending |
| 02-04-T1 | 02-04 | 4 | PLAT-07 | T-02-11 | SQLite overlay starts no DB container; `var/` gitignored | config | `docker compose -f docker-compose.yml -f docker-compose.sqlite.yml config` | ❌ W0 | ⬜ pending |
| 02-04-T2 | 02-04 | 4 | PLAT-07 | T-02-13 | Dialect switch refuses a non-empty target | cli | `DATABASE_URL=sqlite:./target/tmp/switch/octanest.db ./scripts/db-switch-dialect.sh` | ❌ W0 | ⬜ pending |
| 02-04-T3 | 02-04 | 4 | PLAT-08 | T-02-14 | Smoke asserts the dialect that actually served the write | smoke | `make smoke` / `make smoke-mysql` / `make smoke-sqlite` | ❌ W0 | ⬜ pending |
| 02-05-T1 | 02-05 | 5 | PLAT-07, PLAT-08 | T-02-16 | All three dialects proven per PR with no CI secrets | ci | CI job `db-matrix` (`cargo test -p octanest-db --test dialect_probe` per leg) | ❌ W0 | ⬜ pending |
| 02-05-T2 | 02-05 | 5 | PLAT-07 | T-02-18, T-02-19 | Docs name local-only credentials and empty-target-only switching | docs | `grep -q 'sqlite:./var/octanest.db' .env.example && test -f docs/database.md` | ❌ W0 | ⬜ pending |
| 02-05-T3 | 02-05 | 5 | PLAT-07, PLAT-08 | — | Validation contract matches what shipped | doc-gate | `grep -q 'nyquist_compliant: true' .planning/phases/02-multi-db-storage/02-VALIDATION.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. Executor updates Status in 02-05-T3.*

---

## Wave 0 Requirements

- [ ] `crates/octanest-db/src/dialect.rs` unit tests (schemes, mismatch, redaction) — 02-01-T1
- [ ] Migration filename parity check across `migrations/{postgres,mysql,sqlite}/` — 02-02-T1
- [ ] `crates/octanest-db/tests/dialect_probe.rs` gated on `DATABASE_URL` (graceful skip when unset) — 02-02-T3
- [ ] `crates/octanest-db/tests/sqlite_paths.rs` tempdir parent-dir + probe proof (no Docker needed) — 02-02-T3
- [ ] `crates/octanest-api/tests/rpc_db_probe.rs` procedure contract tests — 02-03-T2
- [ ] CI `db-matrix` job for postgres/mysql/sqlite — 02-05-T1
- [ ] `docker compose config` validation for the sqlite overlay in the existing `compose` job — 02-05-T1
- [ ] `scripts/compose-smoke.sh` `system.db_probe` + dialect assertion; `make smoke-mysql` / `make smoke-sqlite` — 02-04-T3

*Existing Phase 1 cargo/CI infrastructure covers runners; Wave 0 adds dialect-specific stubs/jobs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live three-profile Compose bring-up on a workstation | PLAT-08 / ROADMAP #3 | CI proof for D-15 is `cargo db-matrix`; CI compose job stays config-validation only until Phase 22 / PLAT-03 (locked in 02-05-T1) | `make up` / `make up-mysql` / `make up-sqlite` then the matching `make smoke` / `make smoke-mysql` / `make smoke-sqlite` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s for unit path
- [ ] `nyquist_compliant: true` set in frontmatter after plans lock task map

**Approval:** pending
