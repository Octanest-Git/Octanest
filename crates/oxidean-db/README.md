# oxidean-db

Multi-dialect persistence for Oxidean: SQLx migrations and the `Database` API for **PostgreSQL**, **MySQL**, and **SQLite**.

| | |
|--|--|
| **Crate** | `oxidean-db` |
| **Version** | `0.1.0` (workspace) |
| **License** | [MIT](../../LICENSE) |

## Role

- Owns all SQL and dialect branching
- Migrations under `migrations/{postgres,mysql,sqlite}/`
- Called by `oxidean-api` — the API must not fork queries per dialect

## Migrations

Dialect details and Compose overlays: [docs/database.md](../../docs/database.md).

```bash
make up                 # Postgres (default)
make up-mysql
make up-sqlite
```

## Test

```bash
cargo nextest run -p oxidean-db
# or: make test
```

Integration coverage may live under `tests/` and API crate tests that exercise the DB through the API.

## Docs

- [docs/database.md](../../docs/database.md)
- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md)
- [docs/CODE_PRACTICES.md](../../docs/CODE_PRACTICES.md)
