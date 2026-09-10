<!-- generated-by: gsd-doc-writer -->

# @octanest/api-client

Generated TypeScript client for Octanest RPC procedures (specta / rspc-style). Types-only package consumed by `@octanest/web`.

## Scripts

| Script | Command |
|--------|---------|
| Test | `bun run test` (Vitest) |
| Build | no-op (`types-only package`) |

## Regenerate

From the repo root:

```bash
make rpc-gen
# or: cargo run -q -p octanest-api --bin rpc-gen
```

CI enforces sync via `make rpc-sync-check`. See [`docs/API.md`](../../docs/API.md).
