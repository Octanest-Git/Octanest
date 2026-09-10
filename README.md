<!-- generated-by: gsd-doc-writer -->
# Octanest

<p align="center">
  <img src="brand/octanest-mark.png" alt="Octanest" width="128" height="128" />
</p>

A self-hostable GitHub-style social coding platform (git hosting, pull requests, and issues) that runs the same product in the cloud or on your own machines.

**Octanest Cloud** and **self-hosted Octanest** are one codebase: Bun workspaces for the web app and TypeScript packages, plus a Rust Cargo workspace for the API and database layer.

## Installation

Prerequisites: [Bun](https://bun.sh) (see `packageManager` in root `package.json`), Rust/`cargo`, and Docker Compose for the full stack.

```bash
corepack enable
bun install
cargo metadata -q
```

Copy environment defaults for Compose:

```bash
cp .env.example .env
```

## Quick start

1. Install dependencies (`bun install` / `cargo metadata` as above).
2. Bring up the default stack (Traefik on `:80`, web, API, Postgres):

```bash
make up
make smoke
```

3. Or develop locally with Vite + API (see `make dev` for the two-terminal commands):

```bash
make rpc-gen
# terminal 1
OCTANEST_ENV=development API_BIND=127.0.0.1:8080 cargo run -p octanest-api --bin octanest-api
# terminal 2
bun run --filter @octanest/web dev
```

List all Make targets anytime with `make help`.

## Usage examples

### Docker Compose databases

Default stack uses **PostgreSQL**. MySQL and SQLite are first-class overlays:

```bash
make up && make smoke                 # Postgres
make up-mysql && make smoke-mysql     # MySQL profile
make up-sqlite && make smoke-sqlite   # SQLite file under ./var
make down
```

Dialect details, migrations, and switching: [`docs/database.md`](docs/database.md).

### Local auth & email stubs

Exercise SMTP, Resend, WorkOS, and OIDC without cloud secrets:

```bash
cp docs/dev-auth.env.example .env.dev-auth
make up-dev-auth
```

Tear down with `make down-dev-auth`. Full walkthrough: [`docs/dev-auth.md`](docs/dev-auth.md).

### Tests

```bash
make test              # cargo nextest (or cargo test) + bun Vitest
make test-e2e-stack    # Vitest e2e against API + Mailpit/OIDC/stubs
```

## Monorepo layout

| Path | Role |
|------|------|
| `apps/web` | Octane / TanStack Start web app (`@octanest/web`) |
| `packages/api-client` | Generated TypeScript RPC client (`make rpc-gen`) |
| `crates/octanest-api` | Rust API + `rpc-gen` binary |
| `crates/octanest-core` | Shared Rust domain logic |
| `crates/octanest-db` | SQL migrations and DB tooling |

Root tooling: Bun workspaces (`apps/*`, `packages/*`) + Turborepo scripts; Cargo workspace under `crates/`.

## Docs

- [`docs/database.md`](docs/database.md) — Postgres / MySQL / SQLite, migrations, dialect switching
- [`docs/dev-auth.md`](docs/dev-auth.md) — Mailpit, OIDC mock, Resend/WorkOS stubs (`make up-dev-auth`)
