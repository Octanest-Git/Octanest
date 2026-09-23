# PLAN — bun:test + Bun.WebView migration (issue #37)

Status: **in progress** (authoring standard landed; Vitest + Playwright still CI merge gate).  
Track: [PR #38](https://github.com/Octanest-Git/Octanest/pull/38) · docs: [docs/bun-test-webview-poc.md](docs/bun-test-webview-poc.md), [docs/TESTING.md](docs/TESTING.md).

## Goal

Make **`bun:test`** (and **`Bun.WebView`** for live browser) the durable JS/TS test stack for Octanest web + api-client — faster CI, stricter Octane DOM-race isolation — without a third runner. Flip merge gates only when go/no-go criteria below pass.

## Done (do not re-litigate)

| Slice | Landed |
|-------|--------|
| A — unit dual-run | `make test-bun-unit`; `@octanest/web/test-runner` + api-client twin; `octanest-bun-test` condition |
| B — lib integration | `make test-bun-integration` (`theme.integration`); happy-dom preload + cookie jar |
| C — e2e HTTP | `scripts/run-bun-e2e-stack.sh` inside `test-bun-poc-browser` |
| D — WebView (partial) | status, signup, mirror SSH, WorkOS/OIDC, auth.me dedupe, chrome menus, `/new` template |
| Authoring standard | AGENTS / CONTRIBUTING / CODE_PRACTICES / TESTING / DEVELOPMENT + Cursor rule |
| Optional CI | `bun-test-poc` job (`continue-on-error`): unit + lib integration + bench + stack PoC |
| Isolation contract | ephemeral profile, `url: false`, process-per-file, CDP `exceptionThrown` + `DOM_RACE_RE` |

## Remaining work

### 1. Finish WebView ports (Slice D remainder)

Port Playwright commands in `apps/web/e2e/stack-browser/commands.ts` → `apps/web/bun-test/lib/flows.ts` + one process-per-file test under `apps/web/bun-test/browser/`. Prefer `data-testid` / stable CSS; no new `vitest/browser` suites as the primary path.

| Priority | Playwright command / suite | WebView target |
|----------|----------------------------|----------------|
| P0 | `expectForgeRepoPackagesFlow` (`forge-repo`) | `browser/forge-repo.stack.browser.test.ts` |
| P0 | `expectForgeIssuesCrudFlow` / `expectForgeReleasesCrudFlow` | `browser/forge-issues-releases.….ts` |
| P1 | `expectAdminLfsQuotasFlow` (+ admin packages/auth chrome) | `browser/forge-admin.….ts` |
| P1 | `expectForgeSshAndOrgMembersFlow` | `browser/forge-packages-ssh-orgs.….ts` |
| P1 | `expectSettingsProfileAvatarFlow` | `browser/settings-profile-avatar.….ts` |
| P2 | Any leftover auth-ui / mirror coverage still only on Playwright | fold into existing bun files |

Acceptance per flow: green under `make test-bun-poc-browser`; no Octane `insertBefore` / hierarchy `pageerror`; selectors documented in `flows.ts`.

### 2. Integration residuals (Slice B)

Still **Vitest-only** until Bun can load Octane `.tsrx` (or a dedicated loader ships):

- All `src/routes/**` and `src/components/**` `*.integration.test.ts` that mount `.tsrx`
- `src/lib/session-cache.integration.test.ts` (needs real `QueryClientProvider.tsrx`)

Plan options (pick one when unblocking):

1. **Wait** for upstream Octane/Bun loader; keep Vitest happy-dom as residual.
2. **Spike** a Bun plugin that compiles `.tsrx` for tests only (scoped; do not invent a parallel UI runtime).
3. **Narrow** session-cache to a non-render dual-run assertion (QueryClient API only) — weaker than current test; only if product owners accept.

Do not stub `useQuery` into a false green for session-cache.

### 3. Dual-run hygiene

- [ ] `packages/api-client` `test:bun` invoked from CI `bun-test-poc` (or `make test-bun-unit` umbrella) explicitly.
- [ ] Ensure every **new** web unit/gate file imports `@octanest/web/test-runner` (lint/grep gate optional).
- [ ] Keep `vi` from `"vitest"` only where mock hoisting requires it; remap stays under `octanest-bun-test` condition (never plain `bun` — Vitest-under-Bun must not pick the wrong runner).
- [ ] Route-coverage manifest: when a WebView file covers a page, record it alongside Playwright evidence ([TESTING.md](docs/TESTING.md)).

### 4. Stability + metrics (pre-cutover)

- [ ] N≥5 consecutive green `bun-test-poc` CI runs (unit + browser) with flake log.
- [ ] Refresh median wall times in [docs/bun-test-webview-poc.md](docs/bun-test-webview-poc.md) (unit already ~40×; browser stack bring-up still dominates).
- [ ] Confirm zero Chrome profile / user-data leaks into git (`var/`, `tmp/`, ignore rules).
- [ ] Document `BUN_CHROME_PATH` / apt Chromium behavior on `ubuntu-latest`.

### 5. CI cutover (only after go/no-go)

Flip **one slice at a time**; keep Vitest available for residual `.tsrx` until Slice B unblocks.

| Gate today | After cutover (proposed) |
|------------|--------------------------|
| Vitest unit | `make test-bun-unit` required; drop or shrink Vitest unit job |
| Vitest integration | Keep until `.tsrx` dual-run; then bun lib + residual Vitest |
| `test-e2e-stack` Playwright browser | `test-bun-poc-browser` (or renamed `test-bun-e2e-stack`) required; Playwright optional/nightly then remove |
| `bun-test-poc` `continue-on-error` | Promote to blocking, or merge into main web test job |

Go/no-go (all must hold for a slice):

1. Median wall time ≤ Vitest (+ Playwright for browser) for the same flows  
2. Playwright install optional without flake regression on remaining residuals  
3. Zero isolation cross-talk across N sequential process-per-file WebView runs  
4. `Bun.WebView` stable on pinned Bun (`packageManager` / CI bun-version)

### 6. Retire Playwright / Vitest browser (post-cutover)

- [ ] Delete or archive `e2e/stack-browser/commands.ts` Playwright commands once WebView parity exists.
- [ ] Remove `@vitest/browser-playwright` from web deps when no merge-gate browser project remains.
- [ ] Update weighted coverage / e2e checklist scripts to count `bun-test/browser` paths.
- [ ] Scrub docs that still say “install Playwright Chromium” as the default browser path.

### 7. Deferred: Bun-only package-manager cleanup (Workstream 0)

Not required to ship #37 dual-run, but still open:

- Harden root `.gitignore` against competing lockfiles (`yarn.lock`, `package-lock.json`, …)
- Scrub CI/docs “pnpm / Corepack” leftovers
- Migrate isolated [`.railway/`](.railway/) install to `bun install --frozen-lockfile` (keep outside workspaces)

### 8. Explicit non-goals (still)

- Inventing a parallel app structure or React test stack  
- Reviving deleted component Playwright project (**D-QH-03**)  
- Changing coverage-weighted math before WebView checklist parity  
- Forcing forge **registry clients** or stack-preset scaffolds onto Bun  
- Claiming shared e2e SQLite serialization is solved by WebView process-per-file  

## Suggested execution order

1. Port P0 WebView forge flows → green `test-bun-poc-browser`  
2. Port P1 admin / SSH-org / settings avatar  
3. Add api-client bun run to CI PoC job; optional import lint  
4. Collect N≥5 CI stability + refresh metrics doc  
5. Cut over unit gate → then e2e browser gate → then integration when `.tsrx` unblocks  
6. Retire Playwright browser deps; finish Workstream 0 if still desired  

## Day-to-day commands

```bash
make test-bun-unit
make test-bun-integration
make test-bun-poc-browser   # Docker + Chrome; HTTP dual-run + WebView
make bench-bun-poc
make test                   # Vitest merge gate (until cutover)
make test-e2e-stack         # Playwright merge gate (until cutover)
```

## References

- Issue: https://github.com/Octanest-Git/Octanest/issues/37  
- PR: https://github.com/Octanest-Git/Octanest/pull/38  
- Harness: `apps/web/bun-test/`  
- Playwright source of truth for remaining ports: `apps/web/e2e/stack-browser/commands.ts`  
