<!-- generated-by: gsd-doc-writer -->
# Testing

## Test framework and setup

Octanest uses two test stacks:

| Layer | Framework | Config |
|-------|-----------|--------|
| Rust (`crates/*`) | [cargo-nextest](https://nexte.st/) (falls back to `cargo test`) | `.config/nextest.toml` |
| JS / TS (`apps/web`, `packages/api-client`) | Vitest `^5` | `apps/web/vitest.config.ts`, `packages/api-client/vitest.config.ts` |

**Prerequisites**

- Rust toolchain + `cargo-nextest` recommended (`cargo install cargo-nextest --locked`). Without it, `make test` falls back to `cargo test --workspace`.
- Bun `1.4.0` (workspace package manager) and `bun install`.
- For `apps/web` browser projects: Playwright Chromium (`bunx playwright install --with-deps chromium`).
- For full stack e2e: Docker (Mailpit, OIDC mock, Resend/WorkOS HTTP stubs via `docker-compose.dev-auth.yml`).

Root `bun run test` runs Turbo (`turbo run test`), which executes each package’s `test` script (`@octanest/web` and `@octanest/api-client`).

### Rust (nextest)

`.config/nextest.toml` defines:

- **`default`** — local runs; slow-timeout 60s (terminate after 3 periods).
- **`ci`** — `fail-fast = false`, slow-timeout 90s, `test-threads = "num-cpus"`. CI invokes `cargo nextest run --workspace --profile ci`.

Integration-style Rust tests live under `crates/octanest-api/tests/` and `crates/octanest-db/tests/` (plus unit tests colocated in crate sources).

### Vitest projects (`apps/web`)

`apps/web/vitest.config.ts` defines projects. Default `bun run test` / `vitest run` always includes **unit** and **integration**. Projects **e2e-stack** and **e2e-stack-browser** are included only when `E2E_STACK=1` (used by `make test-e2e-stack`).

| Project | Environment | Include pattern | Notes |
|---------|-------------|-----------------|-------|
| `unit` | `node` | `src/**/*.unit.test.ts` | Fast pure logic |
| `integration` | `happy-dom` | `src/**/*.integration.test.{ts,tsx}` | Setup: `src/test/setup-integration.ts` |
| `e2e-stack` | `node` | `e2e/stack/**/*.stack.test.ts` | Only if `E2E_STACK=1`; 60s timeout; no file parallelism |
| `e2e-stack-browser` | Playwright Chromium | `e2e/stack-browser/**/*.stack.browser.test.{ts,tsx}` | Only if `E2E_STACK=1`; 60s timeout |

### `@octanest/api-client`

Package Vitest config: `packages/api-client/vitest.config.ts` (`environment: "node"`). Script: `vitest run`. Tests live next to sources (e.g. `src/index.test.ts`) and cover RPC protocol version headers and client fetch behavior with a mock `fetch`.

## Running tests

### Default suite (`make test`)

```bash
make test
```

1. `cargo nextest run --workspace` if `cargo-nextest` is on `PATH`, else `cargo test --workspace`.
2. `bun run test` → Turbo → Vitest for `@octanest/web` (unit / integration) and `@octanest/api-client`.

Does **not** start Docker stubs or the live API/Vite stack.

### JS-only / filtered Vitest

```bash
bun run test                                    # turbo: all packages with a test script
bun run --filter @octanest/web test             # web: unit + integration
bun run --filter @octanest/web test:unit
bun run --filter @octanest/web test:integration
bun run --filter @octanest/api-client test
```

From `apps/web`:

```bash
bun run test
bun run test:unit
bun run test:integration
```

### Full stack e2e (`make test-e2e-stack`)

```bash
make test-e2e-stack
# equivalent: ./scripts/dev-auth/run-stack-e2e.sh
```

`scripts/dev-auth/run-stack-e2e.sh`:

1. Brings up `docker-compose.dev-auth.yml` (`--profile dev-auth`): Mailpit, OIDC mock, Resend/WorkOS stubs.
2. Builds and runs `octanest-api` on SQLite (default `:18080`), Vite on `:13000`.
3. Sets `E2E_STACK=1` and runs `bun run --filter @octanest/web test:e2e:stack` (`e2e-stack` + `e2e-stack-browser`).
4. Tears down stubs on exit (unless `OCTANEST_E2E_KEEP_STUBS=1`).

Logs land under `var/e2e/` (API/web logs; CI uploads this on failure).

Manual stub stack for interactive use is documented in [dev-auth.md](./dev-auth.md) (`make up-dev-auth` / `make down-dev-auth`).

### Rust-only

```bash
cargo nextest run --workspace
cargo nextest run --workspace --profile ci   # same profile as CI api-rust
cargo test -p octanest-db --test dialect_probe -- --nocapture
```

## Writing new tests

### Web (`apps/web`)

| Kind | Naming | Location |
|------|--------|----------|
| Unit | `*.unit.test.ts` | under `src/` |
| Integration | `*.integration.test.ts(x)` | under `src/` |
| Component e2e | `*.e2e.test.ts(x)` | `e2e/component/` |
| Stack HTTP e2e | `*.stack.test.ts` | `e2e/stack/` |
| Stack browser e2e | `*.stack.browser.test.ts(x)` | `e2e/stack-browser/` |

Shared setup:

- Integration: `src/test/setup-integration.ts`
- Component e2e: `e2e/component/setup.ts`
- Stack: `e2e/stack/setup.ts`, `e2e/stack-browser/setup.ts`

Import from `vitest` explicitly (`globals: false` in the web Vitest config).

### API client

Add `*.test.ts` beside the module under `packages/api-client/src/` (Vitest picks them up with the package default config).

### Rust

- Unit: `#[test]` / `#[tokio::test]` in crate sources.
- Integration: `crates/<crate>/tests/*.rs` (e.g. auth, RPC HTTP/WS, dialect probe). Prefer `cargo nextest` so slow/hanging tests follow `.config/nextest.toml` timeouts.

## Coverage requirements

Weighted forge-core gate (**D-QH-02**). Layers and weights:

| Layer | Weight | What is measured |
|-------|--------|------------------|
| Unit | **25%** (`0.25`) | Web Vitest `--project unit` line coverage (`@vitest/coverage-v8`). Rust lib units via `cargo-llvm-cov` when available (optional today; see residual below). |
| Integration | **40%** (`0.40`) | Web Vitest `--project integration` (happy-dom) line coverage. Rust `tests/` included when llvm-cov runs. |
| E2E / hydration | **35%** (`0.35`) | Interim **checklist score** (fraction of required stack-browser + stack HTTP + smoke script paths present). Not Playwright % coverage yet — expands when the forge matrix (Phase 11.1-03) lands. |

**Weighted score**

```
score = 0.25 * unit + 0.40 * integration + 0.35 * e2e
```

**Initial floor:** bootstrap **`0.65`** (`COVERAGE_WEIGHTED_FLOOR` default in `scripts/coverage-weighted.sh`). Measured baseline after enabling `@vitest/coverage-v8` is ~0.68 (unit ≈61% / integration ≈45% / e2e checklist 1.0). **Ratchet target `0.70`** once integration depth and the forge e2e matrix (11.1-03) land — raise the env default and this doc together. Do not lower without an explicit residual note.

**E2E checklist formula (interim)**

`e2e = present / total` where `total` is the item count in `scripts/coverage-e2e-checklist.sh` (auth stack-browser, SMTP/OIDC stack tests, git/packages smoke scripts). Missing paths lower the score. Forge flows (repo code, issues, releases, packages list, SSH keys, org members) are **not** in the interim list until 11.1-03 adds them.

**Commands**

```bash
make coverage-web          # unit + integration Vitest coverage → var/coverage/*-summary.json
make coverage-rust         # cargo-llvm-cov when installed; otherwise skips with residual note
make coverage-weighted     # collect web (+ optional rust) then run weighted gate
make coverage-contract     # aggregator self-test (under/over floor)
./scripts/coverage-weighted.sh --unit 0.9 --integration 0.9 --e2e 0.9
./scripts/coverage-e2e-checklist.sh
```

From `apps/web`:

```bash
bun run test:coverage:unit
bun run test:coverage:integration
```

Reports: `apps/web/coverage/{unit,integration}/` (`coverage-summary.json`, `lcov.info`). Aggregator copies summaries under `var/coverage/` (gitignored).

**Residual (this wave)**

- Rust `cargo-llvm-cov` is preferred and wired as a Make target + CI install hook, but **CI does not yet fail on Rust coverage numbers** when llvm-cov is too heavy for the job budget — web unit/integration + e2e checklist drive the gate. Revisit when llvm-tools runtime is budgeted.
- Do not revive the removed Playwright component e2e project for coverage (**D-QH-03**).

## CI integration

Workflow: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (`name: CI`). Triggers: push to `main`, all pull requests. Concurrency cancels in-progress runs on the same ref.

| Job | What it runs |
|-----|----------------|
| `api-rust` | Install nextest → `cargo nextest run --workspace --profile ci` |
| `web-octane` | `bun install --frozen-lockfile` → `bun run test` (api-client + web unit/integration) → Turbo build `@octanest/web` |
| `coverage-weighted` | Bun install → `make coverage-contract` → `make coverage-web` → e2e checklist → `scripts/coverage-weighted.sh` (bootstrap floor `0.65`, ratchet target `0.70`); uploads `var/coverage/` + `apps/web/coverage/` on failure |
| `e2e-stack` | Rust + Bun + Playwright → `make test-e2e-stack`; on failure uploads `var/e2e/` as `e2e-stack-logs` |
| `rpc-sync` | `make rpc-sync-check` |
| `compose` | `docker compose … config` for base, MySQL/SQLite overlays, and `docker-compose.dev-auth.yml` |
| `db-matrix` | Matrix `postgres` / `mysql` / `sqlite`: `cargo test -p octanest-db --test dialect_probe -- --nocapture` with matching `DATABASE_URL` / `OCTANEST_DB_DIALECT` |

Default `web-octane` stays fast (no Docker auth stubs). True auth/email path coverage is the separate `e2e-stack` job. The `coverage-weighted` job enforces D-QH-02 without reviving component Playwright.

## Dev-auth stubs (stack e2e)

See [dev-auth.md](./dev-auth.md) for interactive setup. Stack e2e depends on:

| Service | Default local endpoint | Role in tests |
|---------|------------------------|---------------|
| Mailpit | UI `http://127.0.0.1:8025`, SMTP `smtp://127.0.0.1:1025` | Capture SMTP mail |
| OIDC mock | Issuer `http://127.0.0.1:9090/default` | OIDC login without a real IdP |
| HTTP stubs | `http://127.0.0.1:9092` | Resend `POST /emails` + WorkOS AuthKit |

`e2e-stack` proves SMTP→Mailpit, Resend→stub, WorkOS stub login, and OIDC mock login over HTTP. `e2e-stack-browser` exercises signup UI and WorkOS CTA against the live web/API in Chromium.
