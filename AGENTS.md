# Agent guide — Octanest

Short orientation for coding agents and automated contributors. Humans: start with [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Product

Self-hostable GitHub-style forge (git, PRs, issues). **One codebase** for Octanest Cloud and self-hosted. Bun + Turborepo (`apps/*`, `packages/*`) and a Rust Cargo workspace (`crates/*`).

## Stack (do not invent alternatives)

| Layer | Tech |
|-------|------|
| Web UI | **Octane** (`.tsrx`), TanStack Start / Router / Query via `@octanejs/*` |
| API | Rust Axum + typed JSON RPC (`octanest-api`) |
| Domain types | `octanest-core` |
| Persistence | `octanest-db` (Postgres / MySQL / SQLite) |
| TS RPC client | `@octanest/api-client` — **generated** by `make rpc-gen` |

Canonical docs: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/API.md](docs/API.md), [docs/CONFIGURATION.md](docs/CONFIGURATION.md).

## Octane is not React JSX

The web app uses **Octane**, not React as the UI runtime. React-shaped hooks exist; template authoring does **not**.

Before editing UI under `apps/web`:

1. Read the project skill [`.agents/skills/octane/SKILL.md`](.agents/skills/octane/SKILL.md).
2. Prefer official reference: https://octanejs.dev/llms.txt and https://octanejs.dev/docs/differences-from-react
3. Author components in **`.tsrx`** with `function Comp() @{ … }`, `@if` / `@else` (no `@else if`), `@for`, native `onInput` for text fields.
4. Never mix `return (` JSX with Rivet `@{` / `@if` in the same component — that breaks exports/hydration.
5. Server domain data: **TanStack Query** (`@octanejs/tanstack-query`) via `apps/web/src/lib/session-queries.ts` and friends. Local form state stays in `useState`. Do not add Zustand for server/session data.

## Hard boundaries

- **Do not hand-edit** generated client sources as source of truth — change Rust RPC / types, then `make rpc-gen`.
- Dialect SQL lives in `crates/octanest-db` only; API must not branch on DB dialect.
- No production secrets in repo, CI, or docs examples. Use `.env.example` / `docs/dev-auth.env.example`.
- Prefer extending existing patterns over new frameworks, state libraries, or UI kits.

## Commands agents should know

```bash
make help
make rpc-gen                 # regenerate @octanest/api-client
make test                    # Rust nextest + Vitest
make test-e2e-stack          # full auth stack e2e
make up / make smoke         # Compose + health
make rpc-sync-check          # CI gate for client drift
```

See [docs/TESTING.md](docs/TESTING.md) and [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Planning / GSD

Active milestone plans live under `.planning/`. Do not invent roadmap phases; follow `STATE.md` / `ROADMAP.md`. Post-Phase-06 polish (Query session cache, setup auth stack, factory reset) is recorded in `phases/06-self-host-admin-bootstrap/deferred-items.md`.


## Scratch files (`tmp/`)

Use repo-root [`tmp/`](tmp/README.md) for agent and local scratch (screenshots, dumps, one-off scripts, debug logs). The directory is gitignored except `tmp/README.md`. **Never** drop temporary files in the repository root or inside `apps/` / `crates/` / `packages/` source trees.

## Contribution standards

Follow [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/CODE_PRACTICES.md](docs/CODE_PRACTICES.md). Cursor rules under `.cursor/rules/` encode the same expectations for agents.
