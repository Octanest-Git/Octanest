# @octanest/api-client

Generated TypeScript client for Octanest RPC procedures. Types-only package consumed by `@octanest/web`.

| | |
|--|--|
| **Package** | `@octanest/api-client` |
| **Version** | `0.1.0` |
| **License** | [MIT](../../LICENSE) |

**Do not hand-edit as source of truth.** Change Rust RPC / shared types, then regenerate.

## Regenerate

From the repo root:

```bash
make rpc-gen
# or: cargo run -q -p octanest-api --bin rpc-gen
```

CI enforces sync via `make rpc-sync-check` (job `rpc-sync`).

## Scripts

| Script | Command |
|--------|---------|
| Test | `bun run test` (Vitest) |
| Build | no-op (`types-only package`) |

## Docs

- [docs/API.md](../../docs/API.md)
- [docs/DEVELOPMENT.md](../../docs/DEVELOPMENT.md) — RPC codegen sync
- [docs/CODE_PRACTICES.md](../../docs/CODE_PRACTICES.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
