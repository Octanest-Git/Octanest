<!-- generated-by: gsd-doc-writer -->

# @octanest/web

Octane TanStack Start frontend for Octanest (Vite 8, React 19, Tailwind v4).

## Scripts

| Script | Command |
|--------|---------|
| Dev | `bun run dev` (or `bun run --filter @octanest/web dev`) |
| Build | `bun run build` |
| Preview | `bun run preview` |
| Tests | `bun run test` — Vitest unit + integration + e2e-component |
| Stack e2e | `E2E_STACK=1 bun run test:e2e:stack` (requires `make test-e2e-stack` harness) |

Vite proxies `/api/*`, `/uploads`, and `/health` to `OCTANEST_E2E_API_ORIGIN` or `http://127.0.0.1:8080` by default. See root [`docs/DEVELOPMENT.md`](../../docs/DEVELOPMENT.md) and [`docs/TESTING.md`](../../docs/TESTING.md).
