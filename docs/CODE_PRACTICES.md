# Code practices

Conventions for Octanest humans and agents. Complements [CONTRIBUTING.md](../CONTRIBUTING.md) and [AGENTS.md](../AGENTS.md).

## Monorepo boundaries

| Path | Owns | Must not |
|------|------|----------|
| `apps/web` | Octane UI, routes, Vite, Vitest projects | Dialect SQL; hand-written RPC DTOs as source of truth |
| `packages/api-client` | Generated TS client | Manual “fixes” without regenerating from Rust |
| `crates/octanest-api` | HTTP/RPC, auth, email, handlers | DB dialect `if` trees |
| `crates/octanest-core` | Shared domain types / pure logic | I/O, SQL, Axum |
| `crates/octanest-db` | Migrations + `Database` API (all dialects) | Product UI concerns |

Prefer the smallest change that fits an existing pattern. New dependencies need a clear gap (prefer `@octanejs/*` on the web; prefer crates already in the Cargo workspace on the API).

## Versioning & license

Workspace version is **0.1.0** (Cargo `[workspace.package]` and package `package.json` files). License is **MIT** ([LICENSE](../LICENSE)).

## Rust

- Edition 2021; workspace `license` / `version` / `edition`.
- Prefer `Result` + `thiserror` in libraries; avoid `unwrap`/`expect` outside tests.
- Keep auth and privilege checks explicit (`require_verified`, admin gates) — do not weaken for convenience.
- Integration tests under `crates/*/tests/` for cross-module behavior; unit tests colocated when pure.
- Format with `rustfmt`; CI runs `cargo nextest` (see `.config/nextest.toml`).

## TypeScript / Bun

- Bun is the package manager; respect `packageManager` and lockfile (`bun.lock`).
- Prefer existing path aliases (`@/…` in web) over deep relative imports.
- Colocate tests: `*.unit.test.ts`, `*.integration.test.ts`, e2e under `apps/web/e2e/`.
- Do not edit generated `packages/api-client` by hand — change Rust, then `make rpc-gen`.
- **Lint / format / types (web):** `@tsrx/oxc` — `make web-lint` (type-aware `oxlint --deny-warnings`) and `make web-format-check` (`oxfmt`). CI runs both on `web-octane`. Run them before committing web changes. No ESLint/Prettier.

## Octane UI

Full skill: [`.agents/skills/octane/SKILL.md`](../.agents/skills/octane/SKILL.md).

- Author in **`.tsrx`** with Rivet templates (`@{`, `@if`/`@else`, `@for`).
- Do not mix React `return (` JSX with Rivet directives in one component.
- Server/session data: TanStack Query (`apps/web/src/lib/session-queries.ts`). Form fields: local state.
- Text fields: native `onInput`. Anonymous auth pages: SSR loaders, no decorative form skeletons.
- Preserve chrome / brand patterns; do not introduce a second design system.

## RPC & API

- Procedure names and shared types in Rust are authoritative.
- Client regeneration is part of the change: `make rpc-gen` + commit.
- Stable error codes matter for UI (`auth.email_unverified`, etc.) — don’t rename casually.
- Cookies / CORS / Secure flags follow `OCTANEST_ENV` — see [CONFIGURATION.md](CONFIGURATION.md).

## Testing expectations

| Change type | Minimum |
|-------------|---------|
| Pure helper | Unit test |
| UI + Query / session | Integration test (`renderWithQueryClient` where applicable) |
| Auth / RPC contract | Rust integration test and/or stack e2e |
| RPC schema | `make rpc-sync-check` clean |

Details: [TESTING.md](TESTING.md).

## Docs & comments

- Update the nearest package README when a package’s purpose or public commands change.
- Prefer linking to `docs/*` over duplicating long guides.
- Comments explain **why** (threat model, dialect quirk, Octane pitfall), not what the next line does.
- Do not commit planning chatter into product docs unless asked; GSD lives under `.planning/`.

## Security & privacy

- No secrets in git, CI logs, or README examples.
- Privileged actions stay behind verification / admin checks already established in auth phases.
- Factory reset and similar destructive RPCs require explicit confirmation strings — never weaken.
- Uploads and session cookies: follow existing size/type and cookie attribute patterns.
