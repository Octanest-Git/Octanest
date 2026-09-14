---
phase: 20-packages-registry
verified: 2026-09-14T18:05:48Z
status: gaps_found
score: 6/7 must-haves verified
behavior_unverified: 0
overrides_applied: 0
covered_files:
  - .planning/phases/20-packages-registry/20-00-PLAN.md
  - .planning/phases/20-packages-registry/20-00-SUMMARY.md
  - .planning/phases/20-packages-registry/20-01-PLAN.md
  - .planning/phases/20-packages-registry/20-01-SUMMARY.md
  - .planning/phases/20-packages-registry/20-02-PLAN.md
  - .planning/phases/20-packages-registry/20-02-SUMMARY.md
  - .planning/phases/20-packages-registry/20-03-PLAN.md
  - .planning/phases/20-packages-registry/20-03-SUMMARY.md
  - .planning/phases/20-packages-registry/20-04-PLAN.md
  - .planning/phases/20-packages-registry/20-04-SUMMARY.md
  - .planning/phases/20-packages-registry/20-05-PLAN.md
  - .planning/phases/20-packages-registry/20-05-SUMMARY.md
  - .planning/phases/20-packages-registry/20-06-PLAN.md
  - .planning/phases/20-packages-registry/20-06-SUMMARY.md
  - .planning/phases/20-packages-registry/20-07-PLAN.md
  - .planning/phases/20-packages-registry/20-07-SUMMARY.md
  - .planning/phases/20-packages-registry/20-08-PLAN.md
  - .planning/phases/20-packages-registry/20-08-SUMMARY.md
  - .planning/phases/20-packages-registry/20-09-PLAN.md
  - .planning/phases/20-packages-registry/20-09-SUMMARY.md
  - .planning/phases/20-packages-registry/20-10-PLAN.md
  - .planning/phases/20-packages-registry/20-10-SUMMARY.md
  - .planning/phases/20-packages-registry/20-11-PLAN.md
  - .planning/phases/20-packages-registry/20-11-SUMMARY.md
  - .planning/phases/20-packages-registry/20-12-PLAN.md
  - .planning/phases/20-packages-registry/20-12-SUMMARY.md
  - .planning/phases/20-packages-registry/20-CONTEXT.md
  - .planning/phases/20-packages-registry/20-VALIDATION.md
  - apps/web/src/components/packages/delete-version-dialog.tsrx
  - apps/web/src/components/settings/pat-classic-form.tsrx
  - apps/web/src/components/settings/pat-fg-form.tsrx
  - apps/web/src/lib/package-quota-copy.ts
  - apps/web/src/routes/$owner.$repo.packages.integration.test.ts
  - apps/web/src/routes/$owner.$repo.packages.tsrx
  - apps/web/src/routes/$owner.packages.integration.test.ts
  - apps/web/src/routes/$owner.packages.tsrx
  - apps/web/src/routes/admin/packages.integration.test.ts
  - apps/web/src/routes/admin/packages.tsrx
  - apps/web/src/routes/settings/tokens.packages.integration.test.ts
  - apps/web/vite.config.ts
  - crates/octanest-api/src/app.rs
  - crates/octanest-api/src/jobs/schedule.rs
  - crates/octanest-api/src/packages/acl.rs
  - crates/octanest-api/src/packages/auth.rs
  - crates/octanest-api/src/packages/generic.rs
  - crates/octanest-api/src/packages/mod.rs
  - crates/octanest-api/src/packages/npm.rs
  - crates/octanest-api/src/packages/oci.rs
  - crates/octanest-api/src/packages/quota.rs
  - crates/octanest-api/src/packages/rpc.rs
  - crates/octanest-api/src/packages/store.rs
  - crates/octanest-api/src/rpc.rs
  - crates/octanest-api/tests/generic_registry.rs
  - crates/octanest-api/tests/npm_registry.rs
  - crates/octanest-api/tests/oci_registry.rs
  - crates/octanest-api/tests/package_acl.rs
  - crates/octanest-api/tests/package_gc.rs
  - crates/octanest-api/tests/package_quota.rs
  - crates/octanest-api/tests/package_rpc.rs
  - crates/octanest-core/src/package_types.rs
  - crates/octanest-core/src/pat_types.rs
  - crates/octanest-db/migrations/mysql/0015_packages.sql
  - crates/octanest-db/migrations/postgres/0015_packages.sql
  - crates/octanest-db/migrations/sqlite/0015_packages.sql
  - crates/octanest-db/src/packages.rs
  - crates/octanest-db/tests/dialect_packages.rs
  - docker-compose.yml
  - docs/API.md
  - docs/CONFIGURATION.md
  - packages/api-client/src/index.ts
  - scripts/smoke-packages.sh
covered_digest: "v1:sha256:bf26bffeb522172f9a1f53f76b68487cc0393c64128bd4fc5e9e27e89e1aa890"
decision_coverage:
  honored: 16
  total: 16
  not_honored: []
gaps:
  - truth: "Repo packages page lists packages linked to that repository (D-PKG-11)"
    status: failed
    reason: "`$owner.$repo.packages.tsrx` calls `packages.list({ owner })` then filters only `repository_id != null`; it never resolves `$repo` to an id or passes `repository_id`, so packages linked to any repo of the owner appear on every repo packages page."
    artifacts:
      - path: apps/web/src/routes/$owner.$repo.packages.tsrx
        issue: "Client filter ignores route `repo` slug; API `packages.list` by `repository_id` is unused"
      - path: apps/web/src/routes/$owner.$repo.packages.integration.test.ts
        issue: "Vitest only asserts `typeof RepoPackagesPage === 'function'` — does not catch wrong filter"
    missing:
      - "Resolve owner/repo to repository_id (or call packages.list with repository_id)"
      - "Filter/list only packages whose repository_id matches that repository"
      - "Strengthen Vitest to assert repo-scoped listing behavior"
---

# Phase 20: Packages Registry Verification Report

**Phase Goal:** Users can publish and pull OCI, npm, and generic/raw packages scoped to repo/org with the same auth/visibility rules  
**Verified:** 2026-09-14T18:05:48Z  
**Status:** gaps_found  
**Re-verification:** No — initial verification  
**Worktree:** `/home/jesse/wsl-projects/personal/typescript/octanest-wt-phase20` (`feat/execute-20-packages`)  
**Migration:** `0015_packages` (postgres/sqlite/mysql) — confirmed; not 0012/0013

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | User can publish and pull OCI container images from an instance registry scoped to a repo or org | ✓ VERIFIED | `oci.rs` (~954 lines) mounts `/v2`; `oci_registry_auth_push_manifest_blob` + `oci_registry_anonymous_public_pull` pass; digests immutable / tags mutable covered |
| 2 | User can publish and pull npm packages and generic/raw packages from an instance registry scoped to a repo or org | ✓ VERIFIED | `npm.rs` / `generic.rs` substantive; `npm_registry_publish_put_attachment` + `generic_registry_put_file` pass; packument/tarball/dist-tags/search present |
| 3 | Registry packages respect the same auth/visibility rules as their owning repo/org | ✓ VERIFIED | `acl.rs` hybrid ACL∩PAT; `package_acl_*` tests green; handlers call `authorize_*`; classic `repo` alone denied |
| 4 | User can list and delete package versions they are permitted to manage | ✓ VERIFIED | `packages.list` / `packages.deleteVersion` in `rpc.rs` + `rpc.rs` dispatch; `package_rpc_delete_version_admin_confirm` pass; owner UI wires list + type-to-confirm delete |
| 5 | Repo packages page lists packages linked to that repository (D-PKG-11) | ✗ FAILED | Page shows any owner package with non-null `repository_id`; `$repo` unused for filtering; RPC `list_by_repository` unused by UI |
| 6 | Uploads exceeding max blob / owner quota rejected; Admin quota RPC; GC keeps live refs | ✓ VERIFIED | `package_quota_rejects_over_owner_limit` + `package_gc_keeps_shared_blob_and_removes_unref` pass; GC scheduled in `jobs/schedule.rs` |
| 7 | Docs describe `/v2` `/npm` `/generic`, PACKAGES_DIR, PAT scopes; edge smoke skip-ok | ✓ VERIFIED | `docs/CONFIGURATION.md` + `docs/API.md`; Compose Traefik PathPrefix + Vite proxy; `smoke-packages.sh` skip-ok when Compose down |

**Score:** 6/7 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/octanest-db/migrations/*/0015_packages.sql` | Schema packages/versions/blobs/refs/quotas | ✓ VERIFIED | All three dialects; dialect test green |
| `crates/octanest-api/src/packages/{oci,npm,generic,acl,auth,store,rpc,quota}.rs` | Protocol + shared subsystem | ✓ VERIFIED | Substantive; mounted in `app.rs` |
| `crates/octanest-api/tests/{oci,npm,generic}_registry.rs` + `package_*.rs` | Behavioral coverage | ✓ VERIFIED | Named tests pass; no `#[ignore]` stubs left |
| `apps/web/src/routes/$owner.packages.tsrx` | Owner list + delete | ✓ VERIFIED | Query + mutation wired to api-client |
| `apps/web/src/routes/$owner.$repo.packages.tsrx` | Repo-linked list | ✗ HOLLOW | Exists + renders but data filter wrong (see gap) |
| `apps/web/src/components/packages/delete-version-dialog.tsrx` | Type-to-confirm | ✓ VERIFIED | Gates confirm button on `name@version` |
| `apps/web/src/routes/admin/packages.tsrx` | Admin quota UI | ✓ VERIFIED | `adminUsage` / `adminSetQuota` Query/Mutation |
| `apps/web/src/components/settings/pat-*-form.tsrx` | package:read/write + FG | ✓ VERIFIED | Classic + FG PackagesPerm controls |
| `scripts/smoke-packages.sh` | Edge routing smoke | ✓ VERIFIED | Skip-ok without Docker/Compose |
| `packages/api-client` packages.* | Generated RPC client | ✓ VERIFIED | `packages.list/deleteVersion/admin*` present |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `app.rs` | `packages::{oci,npm,generic}::router` | nest `/v2` `/npm` `/generic` | WIRED | Lines 172–176 |
| Protocol handlers | `acl` + `store` + `quota` | authorize + CA commit + check_can_store | WIRED | Grep across oci/npm/generic |
| `rpc.rs` dispatch | `packages::rpc::*` | `packages.list` / `deleteVersion` / admin | WIRED | `crates/octanest-api/src/rpc.rs` |
| Owner packages UI | `@octanest/api-client` | `packagesListQueryOptions` / delete mutation | WIRED | `$owner.packages.tsrx` |
| Repo packages UI | `packages.list` by `repository_id` | Should use repo id | NOT_WIRED | Lists by owner only |
| Traefik / Vite | API | PathPrefix / proxy | WIRED | `docker-compose.yml`, `vite.config.ts` |
| GC job | `quota::gc_unref_blobs` | schedule interval | WIRED | `jobs/schedule.rs` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `$owner.packages.tsrx` | `packages` | `packages.list` RPC → DB | Yes | ✓ FLOWING |
| `$owner.$repo.packages.tsrx` | `packages` | `packages.list({owner})` then wrong client filter | Partial / wrong scope | ⚠️ HOLLOW |
| `admin/packages.tsrx` | `usageQ.data` | `packages.adminUsage` | Yes when lookup set | ✓ FLOWING |
| OCI/npm/generic GET | blob/tarball bytes | CA store + DB refs | Yes (integration tests) | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| OCI push/pull auth | `cargo test -p octanest-api --test oci_registry oci_registry_auth_push_manifest_blob -- --exact` | ok | ✓ PASS |
| npm publish | `… npm_registry_publish_put_attachment -- --exact` | ok | ✓ PASS |
| generic PUT | `… generic_registry_put_file -- --exact` | ok | ✓ PASS |
| ACL publish matrix | `… package_acl_publish_needs_write_and_package_write -- --exact` | ok | ✓ PASS |
| RPC delete confirm | `… package_rpc_delete_version_admin_confirm -- --exact` | ok | ✓ PASS |
| RPC list by repo | `… package_rpc_list_by_repo_link -- --exact` | ok | ✓ PASS |
| Quota reject | `… package_quota_rejects_over_owner_limit -- --exact` | ok | ✓ PASS |
| GC refcount | `… package_gc_keeps_shared_blob_and_removes_unref -- --exact` | ok | ✓ PASS |
| dialect 0014 | `cargo test -p octanest-db --test dialect_packages -- --exact` | ok | ✓ PASS |
| smoke-packages | `./scripts/smoke-packages.sh` | skip (Compose not running) | ? SKIP (skip-ok) |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| N/A | — | No `scripts/*/tests/probe-*.sh` declared for this phase | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| PKG-01 | 00,05,12 | OCI publish/pull | ✓ SATISFIED | oci handlers + `oci_registry` tests |
| PKG-02 | 00,06,07,12 | npm publish/pull (+ dist-tags/search) | ✓ SATISFIED | npm handlers + tests |
| PKG-03 | 00,04,12 | generic/raw | ✓ SATISFIED | generic handlers + tests |
| PKG-04 | 00,02,03,05,11 | Auth/visibility hybrid | ✓ SATISFIED | acl + PAT scopes + tokens UI |
| PKG-05 | 00,08,10,11 | List/delete manage | ⚠️ PARTIAL | Owner list/delete OK; repo-linked UI wrong filter |

Orphaned requirements mapped to Phase 20 but unclaimed by plans: none (PKG-01…05 all claimed).

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts (16/16). Gate non-blocking.

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `oci_registry.rs` | PKG-01 | 8 | 0 | 0 | Behavioral | PASS |
| `npm_registry.rs` | PKG-02 | 7 | 0 | 0 | Behavioral | PASS |
| `generic_registry.rs` | PKG-03 | 5 | 0 | 0 | Behavioral | PASS |
| `package_acl.rs` | PKG-04 | 4 | 0 | 0 | Value | PASS |
| `package_rpc.rs` | PKG-05 | 6 | 0 | 0 | Behavioral | PASS |
| `$owner.packages.integration.test.ts` | PKG-05 | 3 | 0 | 0 | Existence/type only | ⚠️ INSUFFICIENT |
| `$owner.$repo.packages.integration.test.ts` | PKG-05 | 2 | 0 | 0 | Existence only | ⚠️ INSUFFICIENT (missed gap) |
| `admin/packages.integration.test.ts` | PKG-05 | 2 | 0 | 0 | Existence | ⚠️ INSUFFICIENT |
| `tokens.packages.integration.test.ts` | PKG-04 | 3 | 0 | 0 | Existence | ⚠️ INSUFFICIENT |

**Disabled tests on requirements:** 0  
**Circular patterns detected:** 0  
**Insufficient assertions:** 4 web integration files (WARNING — does not alone flip status; repo gap already BLOCKER)

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `dialect_packages.rs` | ~27 | Assert message says `"0012 must define {}"` while checking `0015_packages.sql` | ℹ️ Info | Misleading failure text only; migration id is correct |
| `$owner.$repo.packages.tsrx` | 28–31 | Wrong client-side filter | 🛑 Blocker | D-PKG-11 repo-linked view incorrect |
| Web `*.packages.integration.test.ts` | — | typeof-only stubs left after Wave 0 | ⚠️ Warning | Weak UI regression net |

No unresolved `TBD`/`FIXME`/`XXX` debt markers in packages implementation files.

### Human Verification Required

(Informational while status is `gaps_found` — from VALIDATION.md; re-run UAT after gap closure.)

1. **docker login/push/pull** — Login with PAT-as-password; push `{host}/{owner}/{image}:tag`; anonymous pull if public  
2. **npm publish/install** — Registry `{PUBLIC_ORIGIN}/npm/{owner}/`; publish then install clean  
3. **Type-to-confirm delete UI** — Cancel leaves version; Confirm with `name@version` removes  
4. **Admin packages page** — Usage breakdown readable vs env defaults  

### Gaps Summary

Registry protocols (OCI/npm/generic), hybrid ACL∩PAT, session RPC list/delete, quotas/GC, owner packages UI, admin/tokens UI, docs, and `0015_packages` migration are present, wired, and backed by passing Rust tests. One D-PKG-11 gap blocks phase close: the repo packages route does not scope to the repository in the URL (backend list-by-repo works; UI does not use it). Fix that filter (and preferably strengthen the matching Vitest), then re-verify.

---

_Verified: 2026-09-14T18:05:48Z_  
_Verifier: Claude (gsd-verifier)_
