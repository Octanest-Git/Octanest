# bun:test + Bun.WebView (issue #37)

Authoring standard for Octanest web / api-client tests: **`bun:test`**, with **`Bun.WebView`**
(`backend: "chrome"`) for live-browser stack flows. Vitest + Playwright remain the **CI merge gate**
while dual-run coverage expands; do not add a third runner.

Agents and contributors: see also [TESTING.md](./TESTING.md), [AGENTS.md](../AGENTS.md), and
[CODE_PRACTICES.md](./CODE_PRACTICES.md).

## Commands

```bash
make test-bun-unit          # dual-run all web unit under bun:test
make test-bun-integration   # dual-run lib happy-dom integration (theme)
make test-bun-poc           # thin unit PoC files
make bench-bun-poc          # median wall-time Vitest vs bun:test (3 unit files)
make test-bun-poc-browser   # live stack: e2e HTTP dual-run + WebView browser PoC
```

Artifacts: `var/bun-test-poc/` (bench JSON; gitignored). Local scratch: `tmp/bun-test-poc/`.

## What to write where

| New work | Put it here | Avoid |
|----------|-------------|--------|
| Unit / gate / pure helper | `src/**/*.unit.test.ts` importing `@octanest/web/test-runner` | `from "vitest"` for `describe`/`it`/`expect` |
| Lib DOM helpers | `src/lib/*.integration.test.ts` (dual-run under `preload-web`) | assuming happy-dom cookie APIs without the jar polyfill |
| Route/component `.tsrx` mounts | Vitest happy-dom until Bun has an Octane loader | claiming bun dual-run for suites that import `.tsrx` |
| Stack HTTP | `e2e/stack/*.stack.test.ts` + test-runner import | runner-specific APIs that break Vitest dual-run |
| Stack browser | `apps/web/bun-test/browser/*.stack.browser.test.ts` + `lib/flows.ts` | new Playwright-only `vitest/browser` command suites as the primary path |

## Dual-run coverage

| Slice | bun:test path | Residual (Vitest-only) |
|-------|---------------|------------------------|
| A — unit | `make test-bun-unit` (`src/**/*.unit.test.ts` + gates) | none for pure TS |
| B — integration | `make test-bun-integration` (`src/lib/theme.integration.test.ts`) | route/component `.tsrx` suites; `session-cache.integration` (needs `QueryClientProvider.tsrx`) |
| C — e2e HTTP | `scripts/run-bun-e2e-stack.sh` inside `test-bun-poc-browser` | none for `e2e/stack/*.stack.test.ts` |
| D — browser | `apps/web/bun-test/browser/*.stack.browser.test.ts` | remaining Playwright-only forge flows (admin, issues/releases, packages/SSH/orgs, profile avatar, …) until ported |

Package `@octanejs/tanstack-query` ships `.tsrx` entrypoints Bun cannot load without the Vite Octane plugin. That blocks happy-dom dual-run of anything that renders real `useQuery` / `QueryClientProvider`. Live Chromium via `Bun.WebView` still covers those flows end-to-end.

## Isolation

| Rule | Detail |
|------|--------|
| Ephemeral profile | `dataStore: "ephemeral"` — never commit Chrome user-data dirs |
| Spawn mode | `backend: { type: "chrome", url: false }` — no desktop DevTools attach |
| Process-per-file | Browser files run via `scripts/run-bun-webview-poc.sh` (one Bun process each) |
| Page errors | CDP `Runtime.exceptionThrown` + `DOM_RACE_RE` (Playwright `pageerror` parity). Octane DOM races (`insertBefore` / hierarchy) are the important signal — not console prop warnings. |
| Teardown | `await using` / `close()` then assert; `Bun.WebView.closeAll()` in preload `afterAll` |

The web UI is **Octane** (`.tsrx`), not React. PoC helpers use CSS / `data-testid` / trusted `click`+`type` against Octane `onInput` fields. Do not port React Testing Library patterns here.

## Results (local WSL, 2026-09-23)

### Unit wall time (same 3 files, N=3)

| Runner | median (ms) | min | max |
|--------|-------------|-----|-----|
| Vitest `--project unit` | 561 | 554 | 935 |
| bun:test PoC | 13 | 12 | 13 |

Roughly **40×** faster cold wall time for this slice (startup dominates Vitest).

### Browser PoC (live stack)

| Flow | Result | Notes |
|------|--------|-------|
| `/status` healthy | pass | ~5.5s |
| Local signup UI | pass | CSS `#signup-*` + `type()` / `press("Enter")` |
| Mirror SSH radio (DOM-race gate) | pass | scroll + click / evaluate fallback; no insertBefore |
| WorkOS CTA + OIDC SSO | ported | dual-run under WebView |
| auth.me home dedupe | ported | CDP `Network.requestWillBeSent` count |
| Chrome Create/Account menus | ported | anon hide / signed-in show |
| `/new` template picker | ported | stack overlay + gitignore autofill |
| Forge repo packages | ported | data-testid selectors + navigation |
| Forge issues CRUD | ported | new-issue form + RPC fallback pattern |
| Forge releases CRUD | ported | tag seeding + new-release form + RPC fallback |
| Admin LFS/packages/auth | ported | chrome render + auth readiness checks |
| Forge SSH + org members | ported | SSH key seeding + org settings navigation |
| Settings profile avatar | ported | SSR pages + avatar controls + theme relocation |

Stack bring-up still dominates (API build + Docker stubs + Vite). Runner swap does not remove shared SQLite serialization.

### Stability

Multiple consecutive green stack runs after harness fixes (console.warn filtering, cookie-before-nav, auth.taken race) and complete P0/P1 WebView port coverage. Continue collecting CI stability data — need N≥5 green `bun-test-poc` runs before a gate flip.

## Go / no-go (later cutover)

Flip a slice only when:

1. Median wall time ≤ Vitest (+ Playwright for browser) for the same flows
2. Playwright browser download optional without flake regression
3. Zero isolation cross-talk across N sequential process-per-file runs
4. `Bun.WebView` stable on the pinned Bun version

**Current recommendation:** keep Vitest as merge gate; **author all new tests for bun:test dual-run** and expand WebView coverage. Unit slice is the strongest speed win.

## Layout

```
apps/web/bun-test/
  bunfig.toml
  preload.ts
  preload-unit.ts
  preload-web.ts
  lib/webview-guard.ts
  lib/flows.ts
  unit/*.unit.test.ts
  browser/*.stack.browser.test.ts
```
