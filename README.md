# Octanest

<p align="center">
  <img src="brand/octanest-mark.png" alt="Octanest" width="128" height="128" />
</p>

A GitHub competitor you can also self-host.

**Octanest Cloud** and **self-hosted Octanest** are the same product — social coding, git hosting, pull requests, and issues — run in the cloud or on your own machines (Docker Compose / Railway).

## Development

- **JS package manager:** [Bun](https://bun.sh) via Corepack (`packageManager` in root `package.json`). Fallback: Corepack **pnpm** + Turborepo if Bun is unsuitable.
- **Rust:** Cargo workspace under `crates/` (`octanest-api`, `octanest-core`, `octanest-db`).
- **Web:** `apps/web` (Octane TanStack Start).
- **Generated client:** `packages/api-client`.

```bash
corepack enable
bun install
cargo metadata -q
make help
```

### Local (`make dev`)

Vite proxies `/api/rpc`, `/api/rpc/ws`, and `/healthz` to the API on `:8080` (D-10).

```bash
make rpc-gen
# terminal 1
OCTANEST_ENV=development API_BIND=127.0.0.1:8080 cargo run -p octanest-api --bin octanest-api
# terminal 2
bun run --filter @octanest/web dev
```

### Docker Compose (PLAT-01)

Default stack: **Traefik** (`:80`) + **web** + **api** + **postgres**.

```bash
cp .env.example .env   # local-only password defaults: octanest
make up                # or: docker compose up --build -d
# open http://localhost/
make smoke             # build, wait healthy, curl /, /healthz, RPC system.health, then down
make down
```

Traefik routes `Host(localhost)` → web; `PathPrefix(/api)` and `/healthz` → api (WebSocket upgrades on `/api/rpc/ws`).

**MySQL profile (D-07):**

```bash
docker compose -f docker-compose.yml -f docker-compose.mysql.yml --profile mysql up --build
```

**SQLite (D-07):** no DB Compose service — set `DATABASE_URL=sqlite:./data/octanest.db` on the API only. Full SQLite dialect proof is Phase 2; Phase 1 `octanest-db` is Postgres-shaped.

Do not expose Compose ports to the public internet without auth (auth arrives in later phases).

Planning lives in [`.planning/PROJECT.md`](.planning/PROJECT.md).
