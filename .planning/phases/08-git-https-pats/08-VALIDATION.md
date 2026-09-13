---
phase: "08"
slug: "git-https-pats"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-13"
planned: "2026-09-13"
---

# Phase 08 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: cargo-nextest / `cargo test`; Web: Vitest via Bun |
| **Config file** | workspace Cargo; `apps/web` Vitest (existing) |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(pat_)|test(git_smart)'` |
| **Full suite command** | `make test` (+ `make test-e2e-stack` / `scripts/smoke-git-https.sh` for Traefik/git client) |
| **Estimated runtime** | ~60–180 seconds (quick); full suite longer with e2e |

---

## Sampling Rate

- **After every task commit:** Run `cargo nextest run -p octanest-api -E 'test(pat_)|test(git_smart)'` (or Vitest for UI plans)
- **After every plan wave:** Run `make test` + `make rpc-sync-check` (after 08-08)
- **Before `/gsd-verify-work`:** Full suite green + Smart HTTP e2e through Traefik
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-00-T1 | 00 | 0 | GIT-02, GIT-11 | T-08-01, T-08-02 | Wave 0 RED stubs for PAT + Smart HTTP | integration | `cargo nextest list -p octanest-api -E 'test(pat_) \| test(git_smart)'` | ❌ W0 | ⬜ pending |
| 08-01-T1 | 01 | 0 | GIT-02, GIT-11 | T-08-01, T-08-03 | Wave 0 web stubs tokens + how-to | component | `bun --cwd apps/web exec vitest run src/routes/settings/tokens.integration.test.ts src/components/repo/clone-box.pat.integration.test.ts` | ❌ W0 | ⬜ pending |
| 08-02-T* | 02 | 1 | GIT-02, GIT-11 | T-08-04, T-08-05 | Human locks D-08/D-18/D-21 | checkpoint | DISCUSSION-LOG options recorded | n/a | ⬜ pending |
| 08-03-T1 | 03 | 2 | GIT-11 | T-08-01 | Hash-at-rest schema; no plaintext column | unit/integration | `cargo test -p octanest-db --lib migration_parity && cargo test -p octanest-db --test dialect_pats` | ❌ W0 | ⬜ pending |
| 08-04-T1 | 04 | 3 | GIT-11 | T-08-01 | create/list/revoke; secret once | integration | `cargo nextest run -p octanest-api -E 'test(pat_)'` | ❌ W0 | ⬜ pending |
| 08-04-T2 | 04 | 3 | GIT-02 | T-08-02, T-08-07 | Password reject; cookie ignore; public fetch | integration | `cargo nextest run -p octanest-api -E 'test(git_smart)'` | ❌ W0 | ⬜ pending |
| 08-05-T1 | 05 | 4 | GIT-11 | T-08-06 | FG create + owned-repo bind | integration | `cargo nextest run -p octanest-api -E 'test(pat_)'` | ❌ W0 | ⬜ pending |
| 08-06-T1 | 06 | 4 | GIT-02 | T-08-02, T-08-05 | 401/403/push/last_used | integration | `cargo nextest run -p octanest-api -E 'test(git_smart)'` | ❌ W0 | ⬜ pending |
| 08-06-T2 | 06 | 4 | GIT-02 | T-08-08 | 429 rate limit; unverified push deny | integration | `cargo nextest run -p octanest-api -E 'test(git_smart)'` | ❌ W0 | ⬜ pending |
| 08-07-T1 | 07 | 5 | GIT-02 | T-08-09 | Traefik .git → API | smoke | `rg PathRegexp docker-compose.yml` + `scripts/smoke-git-https.sh` | ❌ | ⬜ pending |
| 08-08-T1 | 08 | 5 | GIT-11 | T-08-10 | rpc-gen; no Bearer claim | codegen | `make rpc-gen && make rpc-sync-check` | ❌ | ⬜ pending |
| 08-09-T1 | 09 | 6 | GIT-11 | T-08-01, T-08-03 | List/nav; Generate gate | component | `bun --cwd apps/web exec vitest run src/routes/settings/tokens.integration.test.ts` | ❌ W0 | ⬜ pending |
| 08-09-T2 | 09 | 6 | GIT-11 | T-08-01 | Revoke confirm Keep token | component | same | ❌ W0 | ⬜ pending |
| 08-10-T1 | 10 | 6 | GIT-11 | T-08-01, T-08-03 | Classic create + reveal | component | same + `bun --cwd apps/web run build` | ❌ W0 | ⬜ pending |
| 08-11-T1 | 11 | 6 | GIT-11 | T-08-06 | FG create UI | component | same | ❌ W0 | ⬜ pending |
| 08-12-T1 | 12 | 7 | GIT-02 | T-08-02 | CloneBox/QuickSetup how-to | component | `bun --cwd apps/web exec vitest run src/components/repo/clone-box.pat.integration.test.ts` | ❌ W0 | ⬜ pending |
| 08-13-T2 | 13 | 7 | GIT-02, GIT-11 | T-08-10 | Phase gate sweep | mixed | `cargo nextest … pat_|git_smart` + `make rpc-sync-check` + web build | ❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-api/tests/pat_rpc.rs` — stubs for GIT-11 (create/list/revoke, email verified) — **08-00**
- [ ] `crates/octanest-api/tests/git_smart_http.rs` — stubs for GIT-02 (anon public, private 401, PAT push, password reject, cookie ignore, scope 403, rate limit 429, unverified push) — **08-00**
- [ ] `crates/octanest-db/migrations/*/0008_pats.sql` + parity — stub **08-00** / green **08-03**
- [ ] `apps/web` Vitest stubs for `/settings/tokens` and CloneBox how-to — **08-01**
- [ ] Compose Traefik `PathRegexp` for `.git` + smoke for `git ls-remote` / `git push` — **08-07**

*Existing nextest/Vitest/`make test` infrastructure covers runners; Wave 0 adds phase-specific stubs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| One-time plaintext PAT reveal + copy | GIT-11 | Browser clipboard / reveal UX | Create token in UI; confirm secret shown once; refresh list — secret gone |
| Confirm dialog before revoke | GIT-11 | Dialog interaction | Revoke flow shows confirm; Keep token leaves token; confirm removes it |
| Clone-box how-to panel readability | GIT-02 | Visual copy | Empty repo / clone box shows username + PAT-as-password + CTA |
| Long-note / long-URL backstops | GIT-11/02 | Visual overflow | List ellipsis+title; FG name truncate; revoke wrap; how-to URL overflow |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending (plans written 2026-09-13; validate-phase after execution)
