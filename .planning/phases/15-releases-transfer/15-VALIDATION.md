---
phase: "15"
slug: "releases-transfer"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-14"
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `15-RESEARCH.md` Validation Architecture. Wave 0 closed by plan `15-00`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo nextest (Rust) + Vitest (web) |
| **Config file** | workspace Cargo / `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(release) \| test(rename) \| test(transfer) \| test(redirect)'` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~90–240 seconds targeted |
| **Phase gate** | Quick run + `cargo test -p octanest-db --lib migration_parity` + `make rpc-sync-check` + `bun --cwd apps/web run build` |

---

## Sampling Rate

- **Per task commit:** focused nextest filter + relevant Vitest file
- **Per wave merge:** `cargo nextest run -p octanest-api -p octanest-db` + web Vitest for touched routes
- **Phase gate:** Full automated gate green before `/gsd-verify-work`

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GIT-14 | Create release for existing tag with notes + asset | API integration | `cargo nextest run -p octanest-api -E 'test(release_create)'` | ❌ Wave 0 |
| GIT-14 | Reject missing tag (`release.tag_missing`) | API integration | same | ❌ Wave 0 |
| GIT-14 | Draft hidden from Read-only / anon | API integration | `cargo nextest run -p octanest-api -E 'test(release_draft)'` | ❌ Wave 0 |
| GIT-15 | Download asset with Read / anon public published | API integration | `cargo nextest run -p octanest-api -E 'test(release_asset)'` | ❌ Wave 0 |
| GIT-15 | Oversized upload rejected | API integration | same | ❌ Wave 0 |
| GIT-16 | Admin rename moves disk + DB; old path redirects | API integration | `cargo nextest run -p octanest-api -E 'test(repo_rename)'` | ❌ Wave 0 |
| GIT-16 | Non-admin rename → soft not_found | API integration | same | ❌ Wave 0 |
| GIT-17 | Admin transfer to user/org + type-confirm | API integration | `cargo nextest run -p octanest-api -E 'test(repo_transfer)'` | ❌ Wave 0 |
| GIT-17 | Issues/LFS associations remain on `repo_id` when tables exist | API integration | `cargo nextest run -p octanest-api -E 'test(repo_transfer_cascade)'` | ❌ Wave 0 |
| GIT-16/17 | New repo at old path supersedes redirect | API integration | `cargo nextest run -p octanest-api -E 'test(redirect_supersede)'` | ❌ Wave 0 |
| UI | Releases tab + settings danger zone | web | `bun --cwd apps/web exec vitest run src/routes/\$owner.\$repo.releases.integration.test.ts src/routes/\$owner.\$repo.settings.rename-transfer.integration.test.ts` | ❌ Wave 0 |
| Dialect | Migration parity for releases + redirects | DB | `cargo test -p octanest-db --test dialect_releases` | ❌ Wave 0 |

---

## Wave 0 Gaps

- [ ] `crates/octanest-api/tests/release_rpc.rs` — GIT-14/15
- [ ] `crates/octanest-api/tests/repo_rename_transfer.rs` — GIT-16/17 + redirects
- [ ] `crates/octanest-db/tests/dialect_releases.rs` — tri-dialect migration parity
- [ ] `apps/web/src/routes/$owner.$repo.releases.integration.test.ts` — tab/routes
- [ ] `apps/web/src/routes/$owner.$repo.settings.rename-transfer.integration.test.ts` — danger-zone rename/transfer confirm
- [ ] None of the above exist today — RED stubs first (match Phases 07–10 Wave 0 style)

---

## Manual / UAT Backstops

- Create release for existing tag; upload asset; download from Releases UI
- Draft visible only to Write+; Admin deletes release
- Admin rename; old `/{owner}/{repo}` and `.git` URLs redirect within retention window
- Admin transfer with type-confirm to user and org; issues/LFS stay reachable on new owner path
