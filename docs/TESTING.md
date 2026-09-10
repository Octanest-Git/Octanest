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

`apps/web/vitest.config.ts` defines projects. Default `bun run test` / `vitest run` always includes **unit**, **integration**, and **e2e-component**. Projects **e2e-stack** and **e2e-stack-browser** are included only when `E2E_STACK=1` (used by `make test-e2e-stack`).

| Project | Environment | Include pattern | Notes |
|---------|-------------|-----------------|-------|
| `unit` | `node` | `src/**/*.unit.test.ts` | Fast pure logic |
| `integration` | `happy-dom` | `src/**/*.integration.test.{ts,tsx}` | Setup: `src/test/setup-integration.ts` |
| `e2e-component` | Playwright Chromium (headless) | `e2e/component/**/*.e2e.test.{ts,tsx}` | Setup: `e2e/component/setup.ts` |
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
2. `bun run test` → Turbo → Vitest for `@octanest/web` (unit / integration / e2e-component) and `@octanest/api-client`.

Does **not** start Docker stubs or the live API/Vite stack.

### JS-only / filtered Vitest

```bash
bun run test                                    # turbo: all packages with a test script
bun run --filter @octanest/web test             # web: unit + integration + e2e-component
bun run --filter @octanest/web test:unit
bun run --filter @octanest/web test:integration
bun run --filter @octanest/web test:e2e         # e2e-component only
bun run --filter @octanest/api-client test
```

From `apps/web`:

```bash
bun run test
bun run test:unit
bun run test:integration
bun run test:e2e
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

No coverage threshold configured (no Vitest/Jest `coverageThreshold`, `.nycrc`, or c8 thresholds in the repo).

## CI integration

Workflow: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (`name: CI`). Triggers: push to `main`, all pull requests. Concurrency cancels in-progress runs on the same ref.

| Job | What it runs |
|-----|----------------|
| `api-rust` | Install nextest → `cargo nextest run --workspace --profile ci` |
| `web-octane` | `bun install --frozen-lockfile` → Playwright Chromium → `bun run test` (api-client + web unit/integration/e2e-component) → Turbo build `@octanest/web` |
| `e2e-stack` | Rust + Bun + Playwright → `make test-e2e-stack`; on failure uploads `var/e2e/` as `e2e-stack-logs` |
| `rpc-sync` | `make rpc-sync-check` |
| `compose` | `docker compose … config` for base, MySQL/SQLite overlays, and `docker-compose.dev-auth.yml` |
| `db-matrix` | Matrix `postgres` / `mysql` / `sqlite`: `cargo test -p octanest-db --test dialect_probe -- --nocapture` with matching `DATABASE_URL` / `OCTANEST_DB_DIALECT` |

Default `web-octane` stays fast (no Docker auth stubs). True auth/email path coverage is the separate `e2e-stack` job.

## Dev-auth stubs (stack e2e)

See [dev-auth.md](./dev-auth.md) for interactive setup. Stack e2e depends on:

| Service | Default local endpoint | Role in tests |
|---------|------------------------|---------------|
| Mailpit | UI `http://127.0.0.1:8025`, SMTP `smtp://127.0.0.1:1025` | Capture SMTP mail |
| OIDC mock | Issuer `http://127.0.0.1:9090/default` | OIDC login without a real IdP |
| HTTP stubs | `http://127.0.0.1:9092` | Resend `POST /emails` + WorkOS AuthKit |

`e2e-stack` proves SMTP→Mailpit, Resend→stub, WorkOS stub login, and OIDC mock login over HTTP. `e2e-stack-browser` exercises signup UI and WorkOS CTA against the live web/API in Chromium.
