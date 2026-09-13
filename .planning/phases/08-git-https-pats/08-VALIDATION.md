---
phase: "08"
slug: "git-https-pats"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-13"
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
| **Full suite command** | `make test` (+ `make test-e2e-stack` for Traefik/git client) |
| **Estimated runtime** | ~60–180 seconds (quick); full suite longer with e2e |

---

## Sampling Rate

- **After every task commit:** Run `cargo nextest run -p octanest-api -E 'test(pat_)|test(git_smart)'`
- **After every plan wave:** Run `make test` + `make rpc-sync-check`
- **Before `/gsd-verify-work`:** Full suite green + Smart HTTP e2e through Traefik
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-W0-01 | 00 | 0 | GIT-11 | T-08-01 | PAT secret never in list; hash at rest | integration | `cargo nextest run -p octanest-api -E 'test(pat_)'` | ❌ W0 | ⬜ pending |
| 08-W0-02 | 00 | 0 | GIT-02 | T-08-02 | Password rejected; cookie ignored; 401/403/429 | integration | `cargo nextest run -p octanest-api -E 'test(git_smart)'` | ❌ W0 | ⬜ pending |
| 08-xx | TBD | TBD | GIT-11 | — | create/list/revoke + email verified | integration | `cargo nextest run -p octanest-api -E 'test(pat_)'` | ❌ W0 | ⬜ pending |
| 08-xx | TBD | TBD | GIT-02 | — | Smart HTTP clone/push with PAT | integration/e2e | `cargo nextest run -p octanest-api -E 'test(git_smart)'` / Traefik smoke | ❌ W0 | ⬜ pending |

*Planner fills concrete Task IDs when PLAN.md files are written. Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-api/tests/pat_rpc.rs` — stubs for GIT-11 (create/list/revoke, email verified)
- [ ] `crates/octanest-api/tests/git_smart_http.rs` — stubs for GIT-02 (anon public, private 401, PAT push, password reject, cookie ignore, scope 403, rate limit 429, unverified push)
- [ ] `crates/octanest-db/migrations/*/0008_pats.sql` + migration parity extension
- [ ] `apps/web` Vitest stubs for `/settings/tokens` and CloneBox how-to panel
- [ ] Compose Traefik `PathRegexp` for `.git` + smoke for `git ls-remote` / `git push`

*Existing nextest/Vitest/`make test` infrastructure covers runners; Wave 0 adds phase-specific stubs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| One-time plaintext PAT reveal + copy | GIT-11 | Browser clipboard / reveal UX | Create token in UI; confirm secret shown once; refresh list — secret gone |
| Confirm dialog before revoke | GIT-11 | Dialog interaction | Revoke flow shows confirm; cancel leaves token; confirm removes it |
| Clone-box how-to panel readability | GIT-02 | Visual copy | Empty repo / clone box shows username + PAT-as-password + CTA |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
