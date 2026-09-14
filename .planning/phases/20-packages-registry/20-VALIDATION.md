---
phase: "20"
slug: "packages-registry"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-14"
---

# Phase 20 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded by plan-phase from 20-RESEARCH.md Validation Architecture. `nyquist_compliant` flips under `/gsd-validate-phase`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: cargo-nextest / `cargo test`; Web: Vitest via Bun |
| **Config file** | `.config/nextest.toml`; `apps/web/vitest.config.ts` |
| **Quick run command** | `cargo nextest run -p octanest-api -E 'test(oci_registry)|test(npm_registry)|test(generic_registry)|test(package_acl)|test(package_rpc)'` |
| **Full suite command** | `make test` (+ `make rpc-sync-check`; `make smoke-packages` skip-ok without Docker) |
| **Estimated runtime** | ~90–240 seconds (quick); full suite longer with e2e/smoke |

---

## Sampling Rate

- **After every task commit:** targeted nextest filter for touched protocol (`oci_registry` / `npm_registry` / `generic_registry` / `package_acl` / `package_rpc`) or Vitest for UI plans
- **After every plan wave:** `cargo nextest run -p octanest-api -p octanest-db` + web package-route Vitest when UI touched + `make rpc-sync-check` after RPC changes
- **Before `/gsd-verify-work`:** `make test` + `make smoke-packages` (skip-ok) green
- **Max feedback latency:** 240 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 20-00-T1 | 00 | 0 | PKG-01..05 | T-20-SC | Wave 0 RED Rust stubs | integration | `cargo nextest list -p octanest-api -E 'test(oci_registry)\|test(npm_registry)\|test(generic_registry)\|test(package_acl)\|test(package_rpc)'` | ❌ W0 | ⬜ pending |
| 20-00-T2 | 00 | 0 | PKG-01..05 | T-20-SC | dialect_packages stub | unit | `cargo nextest list -p octanest-db -E 'test(dialect_packages)'` | ❌ W0 | ⬜ pending |
| 20-01-T1 | 01 | 0 | PKG-05 | T-20-SC | Wave 0 web + smoke stubs | component/smoke | `bun --cwd apps/web exec vitest run --passWithNoTests src/routes/` + `test -f scripts/smoke-packages.sh` | ❌ W0 | ⬜ pending |
| 20-02-T* | 02 | 1 | PKG-04 | T-20-01 | Schema + edge routing + reserved slugs | integration | `cargo test -p octanest-db --test dialect_packages` + `rg PathPrefix.*v2 docker-compose.yml` | ❌ W0 | ⬜ pending |
| 20-03-T* | 03 | 2 | PKG-04 | T-20-02,T-20-03 | Store + ACL∩PAT scopes; cookies ignored | integration | `cargo nextest run -p octanest-api -E 'test(package_acl)'` | ❌ W0 | ⬜ pending |
| 20-04-T* | 04 | 3 | PKG-03 | T-20-04 | Tracer generic PUT/GET/DELETE + mounts | integration | `cargo nextest run -p octanest-api -E 'test(generic_registry)'` | ❌ W0 | ⬜ pending |
| 20-05-T* | 05 | 4 | PKG-01 | T-20-05 | OCI push/pull + Bearer realm | integration | `cargo nextest run -p octanest-api -E 'test(oci_registry)'` | ❌ W0 | ⬜ pending |
| 20-06-T* | 06 | 4 | PKG-02 | T-20-06 | npm publish + tarball install | integration | `cargo nextest run -p octanest-api -E 'test(npm_registry)'` | ❌ W0 | ⬜ pending |
| 20-07-T* | 07 | 5 | PKG-02 | T-20-06 | dist-tags / deprecate / search | integration | `cargo nextest run -p octanest-api -E 'test(npm_registry)'` | ❌ W0 | ⬜ pending |
| 20-08-T* | 08 | 5 | PKG-05 | T-20-07 | Session RPC list/delete Admin | integration | `cargo nextest run -p octanest-api -E 'test(package_rpc)'` | ❌ W0 | ⬜ pending |
| 20-09-T* | 09 | 6 | PKG-01..03 | T-20-08 | Quotas reject + GC refcount-safe | integration | `cargo nextest run -p octanest-api -E 'test(package_acl)|test(generic_registry)'` | ❌ W0 | ⬜ pending |
| 20-10-T* | 10 | 6 | PKG-05 | T-20-07 | Owner/repo packages UI + type-to-confirm | component | `bun --cwd apps/web exec vitest run src/routes/` (packages filters) | ❌ W0 | ⬜ pending |
| 20-11-T* | 11 | 7 | PKG-04,PKG-05 | T-20-02,T-20-08 | Admin quota + tokens package scopes UI | component | Vitest admin + tokens package-scope cases | ❌ W0 | ⬜ pending |
| 20-12-T* | 12 | 8 | PKG-01..05 | T-20-01 | Docs + smoke + phase gate | mixed | `make smoke-packages` + `make rpc-sync-check` + docs rg | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/octanest-api/tests/oci_registry.rs` — PKG-01 stubs (20-00)
- [ ] `crates/octanest-api/tests/npm_registry.rs` — PKG-02 / D-PKG-14 stubs (20-00)
- [ ] `crates/octanest-api/tests/generic_registry.rs` — PKG-03 stubs (20-00)
- [ ] `crates/octanest-api/tests/package_acl.rs` — PKG-04 stubs (20-00)
- [ ] `crates/octanest-api/tests/package_rpc.rs` — PKG-05 stubs (20-00)
- [ ] `crates/octanest-db/tests/dialect_packages.rs` — packages migration parity stub (20-00)
- [ ] `apps/web/src/routes/$owner.packages.integration.test.ts` — owner packages UI stub (20-01)
- [ ] `apps/web/src/routes/$owner.$repo.packages.integration.test.ts` — repo-linked packages stub (20-01)
- [ ] `apps/web/src/routes/admin/packages.integration.test.ts` — Admin quota stub (20-01)
- [ ] `apps/web/src/routes/settings/tokens.packages.integration.test.ts` — package scope tokens stub (20-01)
- [ ] `scripts/smoke-packages.sh` + `make smoke-packages` — edge routing smoke (20-01)

*Existing nextest/Vitest/`make test` infrastructure covers runners; Wave 0 stubs ship first, then later plans turn them green.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `docker login` + `docker push/pull` against `/v2` | PKG-01 | Docker engine may be absent in CI | When Docker available: login with PAT-as-password; push `{host}/{owner}/{image}:tag`; pull anonymous if public |
| `npm publish` / `npm install` against `/npm/{owner}/` | PKG-02 | Client UX + registry config | Configure `@scope:registry` to `/npm/{owner}/`; publish then install in clean dir |
| Type-to-confirm delete dialog | PKG-05 | Browser interaction | Delete flow requires typing `name@version`; Cancel leaves version; Confirm removes |
| Admin quota usage chart readability | PKG-05 / ops | Visual | Admin packages page shows format + package breakdown vs env defaults |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 240s
- [ ] `nyquist_compliant: true` set in frontmatter — owned by `/gsd-validate-phase`

**Approval:** pending
