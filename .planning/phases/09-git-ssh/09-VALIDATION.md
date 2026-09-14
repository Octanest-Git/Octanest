---
phase: "09"
slug: "git-ssh"
status: draft
nyquist_compliant: false
wave_0_complete: true
created: "2026-09-14"
updated: "2026-09-14"
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `09-RESEARCH.md` Validation Architecture; refreshed after 09-00…09-09.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo nextest (Rust) + Vitest (web) |
| **Config file** | workspace Cargo / `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(ssh_key) \| test(git_ssh)'` + `cargo test -p octanest-db --test dialect_ssh_keys` + `bunx vitest run src/routes/settings/ssh-keys.integration.test.ts src/components/repo/clone-box.ssh.integration.test.ts` (cwd `apps/web`) |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~60–180 seconds targeted; full suite longer |

---

## Sampling Rate

- **Per task commit:** targeted nextest filter + relevant vitest file
- **Per wave merge:** `make test`
- **Phase gate:** `make test` + `make smoke-git-ssh` (Compose; skips when Docker missing) + `make rpc-sync-check`

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GIT-04 | add/list/revoke keys; max 25; require_verified; unique fingerprint | integration | `cargo nextest run -p octanest-api -E 'test(ssh_key)'` | ✅ yes |
| GIT-04 | dialect `0009_ssh_keys` CRUD | integration | `cargo test -p octanest-db --test dialect_ssh_keys` | ✅ yes |
| GIT-04 | `/settings/ssh-keys` list + confirm revoke | integration | `bunx vitest run src/routes/settings/ssh-keys.integration.test.ts` | ✅ yes |
| GIT-03 | force user `git`; reject other usernames | integration | `cargo nextest run -p octanest-api -E 'test(git_ssh)'` | ✅ yes |
| GIT-03 | public fetch with key; private non-owner denied (git stderr) | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | push requires verified email | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | only upload/receive-pack; reject shell | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | failed pubkey rate limit | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | Compose TCP + ls-remote/push | smoke | `make smoke-git-ssh` | ✅ yes (needs stack for live run) |
| GIT-03/04 | CloneBox SSH URL + CTA | integration | `bunx vitest run src/components/repo/clone-box.ssh.integration.test.ts` | ✅ yes |

---

## Wave 0 Gaps

- [x] `crates/octanest-api/tests/git_ssh.rs` — greened (auth, ACL, pack allowlist, rate-limit)
- [x] `crates/octanest-db` dialect tests for `0009_ssh_keys`
- [x] `apps/web/src/routes/settings/ssh-keys.integration.test.ts` — greened
- [x] `scripts/smoke-git-ssh.sh` + Makefile target
- [x] `clone-box.integration.test.ts` / `clone-box.ssh.integration.test.ts` — live SSH panel

---

## Manual / UAT Backstops

- Register ed25519 key in settings; CloneBox shows `git@…:owner/repo.git`
- `ssh -p 2222` / git ls-remote against Compose stack (`make up` then `make smoke-git-ssh`)
- Unverified user cannot push; private non-owner denied

## Phase gate notes (09-09)

Automated gate: nextest ssh filters, dialect_ssh_keys, `make rpc-sync-check`, vitest ssh-keys + clone-box.ssh, `bun run build` (apps/web). Live Compose smoke remains an operator/UAT backstop when Docker stack is up.
