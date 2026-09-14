# @octanest/web

Octane + TanStack Start frontend for Octanest (Vite, Tailwind v4). UI is authored primarily as **`.tsrx`** — not React JSX.

| | |
|--|--|
| **Package** | `@octanest/web` |
| **Version** | `0.1.0` |
| **License** | [MIT](../../LICENSE) |

## Stack notes

- Octane skill: [`.agents/skills/octane/SKILL.md`](../../.agents/skills/octane/SKILL.md)
- Agent orientation: [AGENTS.md](../../AGENTS.md)
- Server/session data: TanStack Query via `src/lib/session-queries.ts` (`@octanejs/tanstack-query`)
- RPC: `@octanest/api-client` (regenerate with `make rpc-gen`)

## Scripts

| Script | Command |
|--------|---------|
| Dev | `bun run dev` (or `bun run --filter @octanest/web dev`) |
| Build | `bun run build` |
| Preview | `bun run preview` |
| Tests | `bun run test` — Vitest unit + integration |
| Stack e2e | `E2E_STACK=1 bun run test:e2e:stack` (requires `make test-e2e-stack` harness) |

Vite proxies `/api/*`, `/uploads`, and `/health` to `OCTANEST_E2E_API_ORIGIN` or `http://127.0.0.1:8080` by default.

## Docs

- [docs/DEVELOPMENT.md](../../docs/DEVELOPMENT.md)
- [docs/TESTING.md](../../docs/TESTING.md)
- [docs/CODE_PRACTICES.md](../../docs/CODE_PRACTICES.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
