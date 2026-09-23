# Experimental bun:test + Bun.WebView PoC (issue #37)

Proof of concept for running Octanest web tests on **`bun:test`**, with **`Bun.WebView`**
(`backend: "chrome"`) for stack-browser flows. Vitest + Playwright remain the merge gate.

## Commands

```bash
make test-bun-unit          # dual-run all web unit under bun:test
make test-bun-integration   # dual-run lib happy-dom integration (theme)
make test-bun-poc           # unit dual-run + thin unit PoC files
make bench-bun-poc          # median wall-time Vitest vs bun:test (3 unit files)
make test-bun-poc-browser   # live stack: e2e HTTP dual-run + WebView browser PoC
```

Artifacts: `var/bun-test-poc/` (bench JSON; gitignored). Local scratch: `tmp/bun-test-poc/`.

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

Stack bring-up still dominates (API build + Docker stubs + Vite). Runner swap does not remove shared SQLite serialization.

### Stability

Single consecutive green stack run after harness fixes (console.warn filtering, cookie-before-nav, auth.taken race). Treat as early signal only — need N≥5 CI runs before a gate flip.

## Go / no-go (later cutover)

Flip a slice only when:

1. Median wall time ≤ Vitest (+ Playwright for browser) for the same flows
2. Playwright browser download optional without flake regression
3. Zero isolation cross-talk across N sequential process-per-file runs
4. `Bun.WebView` stable on the pinned Bun version

**Current recommendation:** keep Vitest as gate; expand the PoC suite and dual-run in CI. Unit slice is the strongest speed win.

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
