---
phase: "09"
slug: "git-ssh"
# Lifecycle (#2117): draft → validated. PARTIAL = validated + nyquist_compliant: false
# Wave 0 + API/Vitest gates are complete; full Nyquist remains open until stack-browser SSH e2e exists.
status: validated
nyquist_compliant: false
wave_0_complete: true
created: "2026-09-14"
updated: "2026-09-15"
verified_at: "2026-09-15"
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `09-RESEARCH.md` Validation Architecture; refreshed after 09-00…09-09.
> **Honesty (2026-09-15):** Wave 0 closed and automated req→test map greened; **`nyquist_compliant` remains false** because there is no stack-browser / Playwright e2e for SSH keys, and Compose `smoke-git-ssh` is operator-dependent (not a default CI gate). See `09-VERIFICATION.md`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo nextest (Rust) + Vitest (web) |
| **Config file** | workspace Cargo / `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(ssh_key) \| test(git_ssh)'` + `cargo test -p octanest-db --test dialect_ssh_keys` + `bunx vitest run src/routes/settings/ssh-keys.integration.test.ts src/components/repo/clone-box.ssh.integration.test.ts` (cwd `apps/web`) |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~60–180 seconds targeted; full suite longer |
| **Missing for Nyquist** | stack-browser / `apps/web/e2e` coverage of `/settings/ssh-keys` + CloneBox SSH; CI-enforced live `make smoke-git-ssh` |

---

## Sampling Rate

- **Per task commit:** targeted nextest filter + relevant vitest file
- **Per wave merge:** `make test`
- **Phase gate (automated):** nextest ssh filters + dialect_ssh_keys + `make rpc-sync-check` + vitest ssh-keys + clone-box.ssh + `bun run build` (apps/web) — greened in 09-09
- **Phase gate (ops / residual):** `make smoke-git-ssh` when Compose stack is up (skips exit 0 if Docker missing — **does not prove GIT-03 live transport in CI alone**)

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GIT-04 | add/list/revoke keys; max 25; require_verified; unique fingerprint | integration | `cargo nextest run -p octanest-api -E 'test(ssh_key)'` | ✅ yes |
| GIT-04 | dialect `0009_ssh_keys` CRUD | integration | `cargo test -p octanest-db --test dialect_ssh_keys` | ✅ yes |
| GIT-04 | `/settings/ssh-keys` list + confirm revoke | integration (Vitest DOM) | `bunx vitest run src/routes/settings/ssh-keys.integration.test.ts` | ✅ yes |
| GIT-04 | stack-browser add/list/revoke SSH keys | e2e / browser | — | ❌ **missing** (Phase 11.1 gap) |
| GIT-03 | force user `git`; reject other usernames | integration | `cargo nextest run -p octanest-api -E 'test(git_ssh)'` | ✅ yes |
| GIT-03 | public fetch with key; private non-owner denied (git stderr) | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | push requires verified email | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | only upload/receive-pack; reject shell | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | failed pubkey rate limit | integration | same `git_ssh` filter | ✅ yes |
| GIT-03 | Compose TCP + ls-remote/push | smoke | `make smoke-git-ssh` | ✅ script yes — ⚠ live run needs stack; docker-missing skips |
| GIT-03/04 | CloneBox SSH URL + CTA | integration (Vitest DOM) | `bunx vitest run src/components/repo/clone-box.ssh.integration.test.ts` | ✅ yes |
| GIT-03/04 | browser CloneBox SSH + real git client | e2e / browser | — | ❌ **missing** |

---

## Wave 0 Gaps

- [x] `crates/octanest-api/tests/git_ssh.rs` — greened (auth, ACL, pack allowlist, rate-limit)
- [x] `crates/octanest-db` dialect tests for `0009_ssh_keys`
- [x] `apps/web/src/routes/settings/ssh-keys.integration.test.ts` — greened
- [x] `scripts/smoke-git-ssh.sh` + Makefile target
- [x] `clone-box.integration.test.ts` / `clone-box.ssh.integration.test.ts` — live SSH panel

### Residual Nyquist gaps (keep `nyquist_compliant: false`)

- [ ] Stack-browser (or Playwright) e2e: register/list/revoke SSH key at `/settings/ssh-keys`
- [ ] Stack-browser (or equivalent) proof that CloneBox advertised SSH URL matches a working remote (or document deferred to smoke-only)
- [ ] Optional: CI job that runs `make smoke-git-ssh` against Compose without silent skip counting as green

---

## Manual / UAT Backstops

- Register ed25519 key in settings; CloneBox shows `git@…:owner/repo.git`
- `ssh -p 2222` / git ls-remote against Compose stack (`make up` then `make smoke-git-ssh`)
- Unverified user cannot push; private non-owner denied

## Phase gate notes (09-09)

Automated gate: nextest ssh filters, dialect_ssh_keys, `make rpc-sync-check`, vitest ssh-keys + clone-box.ssh, `bun run build` (apps/web). Live Compose smoke remains an operator/UAT backstop when Docker stack is up.

## Nyquist honesty

| Field | Value | Why |
|-------|-------|-----|
| `status` | `validated` | Plans executed; Wave 0 closed; req→test map accurate; VERIFICATION written |
| `nyquist_compliant` | `false` | Browser e2e sampling for GIT-03/04 UI is absent; smoke is not CI-default proof |
| `wave_0_complete` | `true` | RED stubs greened; infrastructure checklist done |

Do **not** flip `nyquist_compliant: true` until residual gaps above are closed or explicitly accepted with a recorded waiver. Prefer residual-risk documentation over a false green.
