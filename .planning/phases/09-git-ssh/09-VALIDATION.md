---
phase: "09"
slug: "git-ssh"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-14"
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `09-RESEARCH.md` Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo nextest (Rust) + Vitest (web) |
| **Config file** | workspace Cargo / `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(ssh) or test(git_ssh)'` + `bunx vitest run src/routes/settings/ssh-keys.integration.test.ts src/components/repo/clone-box.integration.test.ts` (cwd `apps/web`) |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~60–180 seconds targeted; full suite longer |

---

## Sampling Rate

- **Per task commit:** targeted nextest filter + relevant vitest file
- **Per wave merge:** `make test`
- **Phase gate:** `make test` + `make smoke-git-ssh` (Compose) + `make rpc-sync-check`

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GIT-04 | add/list/revoke keys; max 25; require_verified; unique fingerprint | unit/integration | nextest `ssh_keys` / dialect migration tests | ❌ Wave 0 |
| GIT-04 | `/settings/ssh-keys` list + confirm revoke | integration | vitest settings ssh-keys | ❌ Wave 0 |
| GIT-03 | force user `git`; reject other usernames | unit | nextest ssh auth | ❌ Wave 0 |
| GIT-03 | public fetch with key; private non-owner denied (git error) | integration | nextest ssh pack ACL | ❌ Wave 0 |
| GIT-03 | push requires verified email | integration | nextest | ❌ Wave 0 |
| GIT-03 | only upload/receive-pack; reject shell | unit | nextest | ❌ Wave 0 |
| GIT-03 | Compose TCP + ls-remote/push | smoke | `make smoke-git-ssh` | ❌ Wave 0 |
| GIT-03/04 | CloneBox SSH URL + CTA | integration | vitest clone-box | ⚠️ update placeholder |

---

## Wave 0 Gaps

- [ ] `crates/octanest-api/tests/git_ssh.rs` — RED: auth user, ACL, pack allowlist, rate-limit stubs
- [ ] `crates/octanest-db` dialect tests for `0009_ssh_keys`
- [ ] `apps/web/src/routes/settings/ssh-keys.integration.test.ts` — RED route stubs
- [ ] `scripts/smoke-git-ssh.sh` + Makefile target
- [ ] Update `clone-box.integration.test.ts` expectations once SSH live

---

## Manual / UAT Backstops

- Register ed25519 key in settings; CloneBox shows `git@…:owner/repo.git`
- `ssh -p 2222` / git ls-remote against Compose stack
- Unverified user cannot push; private non-owner denied
