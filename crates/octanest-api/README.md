# octanest-api

Rust HTTP API for Octanest: Axum server, typed JSON RPC (HTTP + WebSocket), auth/session/email, and the `rpc-gen` binary that regenerates `@octanest/api-client`.

| | |
|--|--|
| **Crate** | `octanest-api` |
| **Version** | `0.1.0` (workspace) |
| **License** | [MIT](../../LICENSE) |
| **Bins** | `octanest-api`, `rpc-gen` |

## Role

- Serves `/api/rpc`, `/api/rpc/ws`, `/api/auth/*`, `/health`, uploads
- Owns RPC dispatch, auth stacks (local / WorkOS / OIDC), admin bootstrap, verify/reset
- Depends on [`octanest-core`](../octanest-core/README.md) and [`octanest-db`](../octanest-db/README.md)

Does **not** branch on SQL dialect — that stays in `octanest-db`.

## Run

From repo root:

```bash
OCTANEST_ENV=development API_BIND=127.0.0.1:8080 cargo run -p octanest-api --bin octanest-api
```

Compose: `make up` (see [docs/DEVELOPMENT.md](../../docs/DEVELOPMENT.md)).

## RPC codegen

```bash
make rpc-gen
# cargo run -q -p octanest-api --bin rpc-gen
```

Source of truth: `src/rpc.rs` + shared types in `octanest-core`. CI enforces sync via `make rpc-sync-check`.

## Test

```bash
cargo nextest run -p octanest-api
# or: make test
```

Integration tests live under `tests/`.

## Docs

- [docs/API.md](../../docs/API.md)
- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md)
- [docs/CODE_PRACTICES.md](../../docs/CODE_PRACTICES.md)
- [AGENTS.md](../../AGENTS.md)
