# Octanest

<p align="center">
  <img src="brand/octanest-mark.png" alt="Octanest" width="128" height="128" />
</p>

A GitHub competitor you can also self-host.

**Octanest Cloud** and **self-hosted Octanest** are the same product — social coding, git hosting, pull requests, and issues — run in the cloud or on your own machines (Docker Compose / Railway).

## Development

- **JS package manager:** [Bun](https://bun.sh) via Corepack (`packageManager` in root `package.json`). Fallback: Corepack **pnpm** + Turborepo if Bun is unsuitable.
- **Rust:** Cargo workspace under `crates/` (`octanest-api`, `octanest-core`, `octanest-db`).
- **Web:** `apps/web` (Octane TanStack Start — scaffolded in Phase 1).
- **Generated client:** `packages/api-client`.

```bash
corepack enable
bun install
cargo metadata -q
make help
```

Docker Compose bring-up lands later in Phase 1 (`make up`).

Planning lives in [`.planning/PROJECT.md`](.planning/PROJECT.md).
