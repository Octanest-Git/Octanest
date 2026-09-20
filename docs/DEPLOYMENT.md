<!-- generated-by: gsd-doc-writer -->
# Deployment

Octanest deploys as Docker images behind Traefik on a single HTTP origin. The supported path in-repo is Docker Compose (default Postgres, optional MySQL/SQLite overlays). Cloud hosting is intended to reuse the same images.

Related docs: [CONFIGURATION.md](CONFIGURATION.md), [ARCHITECTURE.md](ARCHITECTURE.md), [database.md](database.md).

## Deployment targets

| Target | Config | Notes |
|--------|--------|-------|
| Docker Compose (default) | `docker-compose.yml` | Traefik `:80` + `web` + `api` + Postgres 16 |
| MySQL overlay | `docker-compose.yml` + `docker-compose.mysql.yml` (`--profile mysql`) | Overrides API `DATABASE_URL` / dialect; Postgres service may still start |
| SQLite overlay | `docker-compose.yml` + `docker-compose.sqlite.yml` | No DB container; file under `${OCTANEST_SQLITE_HOST_DIR:-./var}` → `/app/var` (Compose v2.24+ for `!reset`) |
| Dev-auth stubs (local only) | `docker-compose.dev-auth.yml` (`--profile dev-auth`) | Mailpit / OIDC mock / HTTP stubs — not a production stack |
| API image | `crates/octanest-api/Dockerfile` | Multi-stage Rust release binary `octanest-api`, listens `0.0.0.0:8080` |
| Web image | `apps/web/Dockerfile` | Bun build of `@octanest/web`, runs `bun run preview` on `:3000` |
| Traefik extras | `deploy/traefik/` | Optional static YAML notes; default routing is Compose labels + Traefik CLI flags |
| Octanest Cloud (Railway-class) | `.railway/railway.ts` + `deploy/cloud/` | Same api/web Dockerfiles; managed Postgres; Caddy file gateway (no Docker socket) |

### Octanest Cloud (Railway)

Cloud deploys the **same** `crates/octanest-api/Dockerfile` and `apps/web/Dockerfile` images as Compose (D-CLOUD-01). Default database is **managed Postgres** (D-CLOUD-02). HTTP ingress is a **file-configured Caddy** gateway under `deploy/cloud/` — not Traefik’s Docker provider (D-CLOUD-03).

| Railway service | Role |
|-----------------|------|
| `postgres` | Managed Postgres plugin |
| `api` | Forge API + volumes under `/var/*` |
| `web` | SPA (`bun run preview`) — private |
| `gateway` | Public HTTPS edge (`deploy/cloud/Caddyfile`) |

#### Environments

| Environment | Role | Deploy trigger |
|-------------|------|----------------|
| `preview` | Base for Railway **PR Environments** (ephemeral per-PR copies) | Autodeploy off |
| `staging` | Always-on integration | Autodeploy from `main` + Wait for CI |
| `production` | Live | Autodeploy off; promote via GitHub Action |

PR Environments inherit from **`preview`** (Project Settings → Environments). Only authors who are Railway project members with GitHub linked get automatic PR deploys. Invite future collaborators as project **Viewer** (or higher) and have them link GitHub. Keep Bot PR Environments off unless you want Dependabot-style previews.

Focused PR Environments (optional but recommended for this monorepo): enable in project settings, then set watch paths on `api` / `web` / `gateway` (see [`.railway/README.md`](../.railway/README.md)).

#### Operator steps (D-CLOUD-07 — human apply only)

1. Create/link a Railway project: `railway login` → `railway link` (token stays in the operator environment — never commit; never expose to fork PRs).
2. Install IaC SDK: `cd .railway && npm ci` (isolated from the Bun monorepo).
3. For each of `preview`, `staging`, and `production`: link that environment, then `make cloud-plan` (wraps `railway config plan`).
4. In the Railway dashboard, set per-environment vars:
   - `OCTANEST_ENV=preview` \| `staging` \| `production`
   - **`web`:** `OCTANEST_VITE_ALLOWED_HOSTS` — comma-separated Vite Host allowlist. Cloud example: `.up.railway.app,octanest.jereko.dev` (leading `.` allows all Railway `*.up.railway.app` PR/gateway hosts; add each custom apex/host you terminate on the gateway). Without this, `vite preview` returns “Blocked request. This host is not allowed.” See [CONFIGURATION.md](CONFIGURATION.md).
   - Optional: `OCTANEST_ADMIN_EMAIL` / `OCTANEST_ADMIN_PASSWORD`, email/SSO keys
   - **`api`:** `OCTANEST_ACTIONS_SECRETS_KEY` — unique per environment (AES-256-GCM for Actions secrets, mirror credentials, and inbound webhook secrets). Required; encrypt fails closed if unset. Generate with `openssl rand -base64 32`. PR Environments inherit this from **preview**, so set it on preview before opening PRs that exercise mirrors/Actions secrets.
   - IaC wires `OCTANEST_PUBLIC_ORIGIN`, `OCTANEST_CORS_ORIGINS`, and `OCTANEST_SSH_HOST` from the **gateway** public domain (`https://${{gateway.RAILWAY_PUBLIC_DOMAIN}}`). Do not `preserve()` those on preview/PR copies — stale preview hosts break setup/CORS. Production custom domains still need the gateway service domain (or override) to match the browser URL. At runtime, a stale `*.up.railway.app` `OCTANEST_PUBLIC_ORIGIN` is replaced with `RAILWAY_SERVICE_GATEWAY_URL` / `RAILWAY_PUBLIC_DOMAIN`; custom domains are not overridden.
5. Attach a Railway-provided (or custom) domain to **`gateway`** in each environment (required so PR Environments get automatic preview URLs).
6. Review the plan, then **only with explicit approval**: `railway config apply`.
7. **Deploy policy after apply:**
   - `staging`: GitHub branch `main`, autodeploy on, Wait for CI on
   - `preview` and `production`: GitHub connected, autodeploy **off**
8. **Migrations:** IaC sets `OCTANEST_AUTO_MIGRATE=true` for non-production (preview, staging, and PR Environments cloned from preview) so ephemeral databases get schema on boot. **Production** stays `false` — for first boot, temporarily set `OCTANEST_AUTO_MIGRATE=true` on `api` (or run a one-off migrate job), then return to `false`.
9. Confirm `GET https://<domain>/health` and browser `/`.

#### Promote / rollback production

Do **not** use Railway Environment Sync to promote: Sync copies service **variables** and can overwrite production origins (`OCTANEST_PUBLIC_ORIGIN`, `OCTANEST_CORS_ORIGINS`, `OCTANEST_ENV`, etc.) with staging values.

**Primary path — GitHub Action “Production deploy”** ([`.github/workflows/production-deploy.yml`](../.github/workflows/production-deploy.yml)):

1. Validate the commit on **staging** (autodeploys from `main` + Wait for CI).
2. Repo → **Actions** → **Production deploy** → **Run workflow**:
   - `action=promote` — deploys a commit SHA to production `api`, `web`, and `gateway` (default SHA = `main` HEAD; optional override). Requires a successful **CI** run on that SHA. Does not mutate Railway variables.
   - `action=rollback` — Railway `deploymentRollback` to the previous `canRollback` deployment on each of those services (restores that deployment’s image; Railway may also restore that deployment’s custom variables).
   - `dry_run=true` — resolves SHA / CI / rollback targets and prints the plan only (no deploys, rollbacks, or health probe).
3. Confirm `GET https://octanest.jereko.dev/health` (skipped when `dry_run=true`).

**One-time GitHub setup:** On Environment [`Octanest / production`](https://github.com/Octanest-Git/Octanest/settings/environments/22303549290/edit), add secret `RAILWAY_TOKEN` (token with deploy rights), optionally enable required reviewers. Local dry-run: `scripts/railway-production-deploy.sh list` / `promote <sha> --dry-run` / `rollback --dry-run`.

**Avoid:** Sync staging → production unless you carefully reject variable diffs in staged changes.

**Volumes (D-CLOUD-04):** `forge-data` mounts at `/var` on `api` (repos, lfs, packages, release-assets, uploads, ssh host keys as subdirs — same paths as Compose). Each environment has its own volume and database.

**Git (D-CLOUD-08):** HTTPS Smart HTTP through the gateway is the always-on cloud clone path. Optional TCP publish for `OCTANEST_SSH_PORT` (2222) on `api` when the host supports it — do not route SSH through the HTTP gateway.

See [`.railway/README.md`](../.railway/README.md) and [`deploy/cloud/README.md`](../deploy/cloud/README.md).

There is no `railway.json` / `railway.toml` (deprecated). Production tokens never land in this repository.

### Compose bring-up

```bash
# Default (Postgres)
make up
# or: docker compose -f docker-compose.yml up --build -d

# MySQL
make up-mysql
# or: docker compose -f docker-compose.yml -f docker-compose.mysql.yml --profile mysql up --build -d

# SQLite (writes .env.sqlite with OCTANEST_SQLITE_HOST_DIR via scripts/sqlite-host-dir.sh)
make up-sqlite
# or: docker compose -f docker-compose.yml -f docker-compose.sqlite.yml up --build -d

make down          # default stack
make down-mysql
make down-sqlite
make logs          # follow default compose logs
```

### Actions runner (optional)

Octanest does **not** run CI jobs inside the API. Operators attach compute via the official runner image (`docker/octanest-runner`, act_runner lineage).

```bash
# Instance / Admin registration token — never commit real values
export OCTANEST_RUNNER_REGISTRATION_TOKEN=...
# Prefer a hostname job containers can reach (not 127.0.0.1 from nested Docker)
export OCTANEST_COMPOSE_PUBLIC_ORIGIN=http://localhost

docker compose --profile actions up -d --build runner
```

Standalone `docker run` instructions: [`docker/octanest-runner/README.md`](../docker/octanest-runner/README.md). Smoke: `bash scripts/smoke-actions.sh`.

Network name: `octanest_octanest`. Avatar uploads bind `./var/uploads` → `/var/uploads` on `api`.

### Forge volumes & ports

Default Compose publishes HTTP via Traefik and raw TCP for Git-over-SSH. Persist forge data on the host under `./var/` (create as needed; gitignored).

| Host | Container / role | Notes |
|------|------------------|--------|
| `:80` | Traefik → web + API | Browser origin `http://localhost` |
| `:2222` | API SSH (`OCTANEST_SSH_PORT`) | Raw TCP — **not** routed through Traefik. Disable with `OCTANEST_SSH_ENABLED=false` |
| `./var/repos` | `/var/repos` | Bare git repositories |
| `./var/lfs` | `/var/lfs` (`OCTANEST_LFS_DIR`) | Git LFS object store |
| `./var/packages` | `/var/packages` (`OCTANEST_PACKAGES_DIR`) | OCI / npm / generic blobs |
| `./var/release-assets` | `/var/release-assets` (`OCTANEST_RELEASE_ASSETS_DIR`) | Release asset files (distinct from LFS) |
| `./var/ssh` | `/var/ssh` (`OCTANEST_SSH_HOST_KEY_DIR`) | SSH host keys (TOFU across restarts) |
| `./var/uploads` | `/var/uploads` | Avatars / uploads |
| `./var/actions-logs` | `/var/actions-logs` (`OCTANEST_ACTIONS_LOG_DIR`) | Actions job logs (distinct from repos/LFS/packages) |

Env knobs: [CONFIGURATION.md](CONFIGURATION.md).

### Traefik routing

Traefik `v3.3` is configured in Compose (`--providers.docker=true`, `--providers.docker.exposedbydefault=false`, entrypoint `web` on `:80`). Dashboard is off.

| Router | Rule | Priority | Backend |
|--------|------|----------|---------|
| `api-git` | `Host(\`localhost\`) && PathRegexp(\`^/[^/]+/[^/]+\\.git\`)` | 110 | `api:8080` (Smart HTTP) |
| `api-packages` | `Host(\`localhost\`) && (PathPrefix(\`/v2\`) \|\| PathPrefix(\`/npm\`) \|\| PathPrefix(\`/generic\`))` | 110 | `api:8080` (registries) |
| `api` | `Host(\`localhost\`) && (PathPrefix(\`/api\`) \|\| PathPrefix(\`/uploads\`) \|\| Path(\`/health\`))` | 100 | `api:8080` |
| `web` | `Host(\`localhost\`)` | 1 | `web:3000` |

Both services set `traefik.enable=true` and `traefik.docker.network=octanest_octanest`. Browser traffic uses `http://localhost` (no TLS in default Compose). Git SSH stays on host `:2222` and does not use Traefik.

For **Octanest Cloud**, path priorities are mirrored in [`deploy/cloud/Caddyfile`](../deploy/cloud/Caddyfile) with a public Host catch-all (platform TLS). Do not run Docker-socket Traefik on Railway.

### Dialect overlays

- **Postgres (default):** `DATABASE_URL=postgres://octanest:octanest@postgres:5432/octanest`; volume `postgres_data`.
- **MySQL:** profile `mysql`, image `mysql:8.4`; API gets `MYSQL_DATABASE_URL` (default `mysql://octanest:octanest@mysql:3306/octanest`) and `OCTANEST_DB_DIALECT=mysql`. Default Postgres still starts unless stopped.
- **SQLite:** `postgres` moved to profile `postgres` (so it does not start); `depends_on` reset; `DATABASE_URL=sqlite:/app/var/octanest.db`; host dir via `OCTANEST_SQLITE_HOST_DIR` (WSL + `docker.exe` may need a Windows-native path from `scripts/sqlite-host-dir.sh`).

## Build pipeline

CI (`.github/workflows/ci.yml`, triggers: push to `main`, pull requests) **validates** Compose and builds/tests artifacts; it does **not** push images or deploy.

1. `api-rust` — `cargo nextest run --workspace --profile ci`
2. `web-octane` — `bun install --frozen-lockfile`, Vitest, `bunx turbo run build --filter=@octanest/web`
3. `e2e-stack` — `make test-e2e-stack` (API + Mailpit/OIDC/stubs + web)
4. `rpc-sync` — `make rpc-sync-check`
5. `compose` — `docker compose … config` for default, MySQL, SQLite, and dev-auth files
6. `db-matrix` — migrate + `dialect_probe` for postgres / mysql / sqlite

Local image build happens on `docker compose … up --build` (or `make up` / `up-mysql` / `up-sqlite`). API Dockerfile: `cargo build --release -p octanest-api --bin octanest-api`. Web Dockerfile: `bun run --filter @octanest/web build`, then `preview`. Cloud uses the same Dockerfiles via Railway `DOCKERFILE` builder (no mandatory registry publish in this phase).

## Environment setup

Production-like Compose should copy [`.env.example`](../.env.example) to `.env` and set at least:

| Variable | Production guidance |
|----------|---------------------|
| `DATABASE_URL` | Real DB URL matching dialect |
| `OCTANEST_ENV` | Not `development`/`dev` (Compose default is `compose`) |
| `OCTANEST_CORS_ORIGINS` | Comma-separated public browser origins (**required** when env is not `development`/`dev`) |
| `OCTANEST_AUTO_MIGRATE` | Prefer `false` for prod-like; run `make db-migrate` |
| `OCTANEST_PUBLIC_ORIGIN` | Browser-facing origin for SSO callbacks behind Traefik |
| Auth / email secrets | `WORKOS_*`, `OCTANEST_OIDC_*`, `OCTANEST_RESEND_API_KEY` / `OCTANEST_SMTP_URL` as needed |

Full variable table and defaults: [CONFIGURATION.md](CONFIGURATION.md). Cloud secrets live in the Railway dashboard / `preserve()` — see Octanest Cloud section above.

## Smoke targets

`scripts/compose-smoke.sh` brings the stack up with `--wait`, then checks Traefik-facing URLs and RPC. Env knobs: `OCTANEST_SMOKE_URL` (default `http://localhost`), `COMPOSE_FILES`, `COMPOSE_PROFILES`, `EXPECT_DIALECT`, optional `OCTANEST_SQLITE_HOST_DIR`.

| Make target | Expectation |
|-------------|-------------|
| `make smoke` | Default Compose; `EXPECT_DIALECT=postgres` |
| `make smoke-mysql` | MySQL overlay + profile; dialect `mysql` |
| `make smoke-sqlite` | SQLite overlay; dialect `sqlite` |
| `make smoke-actions` | Runner Dockerfile/Compose + optional Docker build; `/api/actions` protocol reachability when stack up (skip-ok without Docker) |

Checks performed:

1. `docker compose … config`
2. `docker compose … up --build -d --wait`
3. `GET /` (web)
4. `GET /health` (API via Traefik)
5. RPC `system.health` (`Octanest-RPC-Version: 1`)
6. RPC `system.db_probe` twice — dialect match and increasing `probe_count`

Trap runs `docker compose … down --remove-orphans` on exit.

## Rollback procedure

No automated rollback is defined in CI or platform config files.

1. Stop the current stack: `make down` (or the matching `down-mysql` / `down-sqlite`).
2. Redeploy a known-good revision: check out the previous git tag/commit, then `make up` (or rebuild with the prior image tags if you publish images externally).
3. Confirm with `make smoke` (or the dialect-specific smoke target) and `GET /health` / `system.health`.

**Octanest Cloud:** Redeploy the previous successful Railway deployment for `gateway` / `api` / `web`, or check out a known-good git revision and redeploy after `make cloud-plan` review.

## Monitoring

In-repo observability is health-oriented only — no Sentry, Datadog, New Relic, or OpenTelemetry packages detected.

| Signal | How |
|--------|-----|
| API process health | `GET /health` (Compose/Dockerfile healthchecks use `curl` to `127.0.0.1:8080/health`) |
| Web process health | `GET /` on `:3000` |
| Edge health | `GET http://localhost/health` through Traefik |
| DB reachability | RPC `system.db_probe` (used by smoke) |
| App status UI | Web `/status` surfaces API reachability (see product copy) |
| Logs | `make logs` / `docker compose logs -f` |
| Cloud logs | Railway service logs for `gateway` / `api` / `web` |

Platform metrics/alerting are operator-chosen (not defined in-repo).
