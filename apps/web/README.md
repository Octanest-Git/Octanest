# @oxidean/web

Octane + TanStack Start frontend for Oxidean (Vite, Tailwind v4). UI is authored primarily as **`.tsrx`** — not React JSX.

|             |                      |
| ----------- | -------------------- |
| **Package** | `@oxidean/web`       |
| **Version** | `0.1.0`              |
| **License** | [MIT](../../LICENSE) |

## Stack notes

- Octane skill: [`.agents/skills/octane/SKILL.md`](../../.agents/skills/octane/SKILL.md)
- Agent orientation: [AGENTS.md](../../AGENTS.md)
- Server/session data: TanStack Query via `src/lib/session-queries.ts` (`@octanejs/tanstack-query`)
- RPC: `@oxidean/api-client` (regenerate with `make rpc-gen`)

## Scripts

| Script    | Command                                                                       |
| --------- | ----------------------------------------------------------------------------- |
| Dev       | `bun run dev` (or `bun run --filter @oxidean/web dev`)                        |
| Build     | `bun run build`                                                               |
| Preview   | `bun run preview`                                                             |
| Lint      | `bun run lint` — type-aware `oxlint` (`@tsrx/oxc`)                            |
| Format    | `bun run format` / `bun run format:check` — `oxfmt`                           |
| Tests     | `bun run test` — Vitest unit + integration                                    |
| Stack e2e | `E2E_STACK=1 bun run test:e2e:stack` (requires `make test-e2e-stack` harness) |

Vite proxies `/api/*`, `/uploads`, and `/health` to `OXIDEAN_E2E_API_ORIGIN` or `http://127.0.0.1:8080` by default.

## Docs

- [docs/DEVELOPMENT.md](../../docs/DEVELOPMENT.md)
- [docs/TESTING.md](../../docs/TESTING.md)
- [docs/CODE_PRACTICES.md](../../docs/CODE_PRACTICES.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
