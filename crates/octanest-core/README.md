# octanest-core

Shared Rust domain types and pure logic for Octanest. Consumed by `octanest-api` and `octanest-db` (and surfaced to TypeScript via `rpc-gen`).

| | |
|--|--|
| **Crate** | `octanest-core` |
| **Version** | `0.1.0` (workspace) |
| **License** | [MIT](../../LICENSE) |

## Role

- Auth and public DTOs (e.g. `auth_types`) used across the API surface
- No I/O, SQL, or HTTP — keep this crate dependency-light (`serde` / `serde_json`)

When you change types that cross the RPC boundary, regenerate the client:

```bash
make rpc-gen
```

## Test

```bash
cargo nextest run -p octanest-core
# or: make test
```

## Docs

- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md)
- [docs/API.md](../../docs/API.md)
- [docs/CODE_PRACTICES.md](../../docs/CODE_PRACTICES.md)
