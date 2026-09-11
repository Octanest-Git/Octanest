<!-- generated-by: gsd-doc-writer -->
# Configuration

Octanest is configured primarily through environment variables. Canonical templates live in [`.env.example`](../.env.example) (Compose / local API) and [`docs/dev-auth.env.example`](dev-auth.env.example) (local auth/email stubs). Copy templates into untracked files such as `.env` or `.env.dev-auth` — never commit secrets.

Related docs: [database.md](database.md), [dev-auth.md](dev-auth.md).

## Environment variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Optional* | _(unset)_ | Connection URL. Scheme selects dialect: `postgres://` / `postgresql://`, `mysql://`, or `sqlite:` (also `sqlite://`, `sqlite::memory:`). `mariadb://` is not supported. If unset, the API boots without a DB pool. |
| `OCTANEST_DB_DIALECT` | Optional | _(inferred from URL)_ | Explicit dialect: `postgres`, `mysql`, or `sqlite`. When set, must agree with `DATABASE_URL` or boot exits. |
| `OCTANEST_AUTO_MIGRATE` | Optional | `true` | When `true`/`1`, run sqlx migrations on API boot if a database is configured. Set `false` for prod-like deploys and run `make db-migrate`. |
| `OCTANEST_ENV` | Optional | `development` | Runtime mode. Affects CORS, session cookie `Secure`, and whether `OCTANEST_OIDC_ALLOW_INSECURE` is honored. Common values: `development`, `dev`, `compose`, `production`. |
| `OCTANEST_CORS_ORIGINS` | Conditional | _(none)_ | Comma-separated browser origins. **Required** when `OCTANEST_ENV` is not `development` or `dev` (e.g. `compose`). Ignored for CORS allowlisting in `development`/`dev` (request origin is mirrored). |
| `API_BIND` | Optional | `0.0.0.0:8080` | Listen address for `octanest-api`. Host `make` workflows often use `127.0.0.1:8080`. |
| `OCTANEST_RPC_VERSION` | Optional | `1` (Compose) | Documented / passed through Compose. The generated TS client embeds `RPC_VERSION = 1` and sends header `Octanest-RPC-Version`. |
| `RUST_LOG` | Optional | `info` (Compose) | `tracing` / `EnvFilter` log directives. |
| `OCTANEST_PUBLIC_ORIGIN` | Optional | `http://localhost:8080` (API fallback when unset) | Browser-facing origin for **verify/reset magic links** and SSO redirect URIs. Trailing slash is stripped. Compose defaults to `http://localhost` via `OCTANEST_COMPOSE_PUBLIC_ORIGIN` so a host Vite value (`http://localhost:3000`) in `.env` does not poison container mail. |
| `OCTANEST_COMPOSE_PUBLIC_ORIGIN` | Optional (Compose) | `http://localhost` | Sets `OCTANEST_PUBLIC_ORIGIN` inside the API container (`make up` / `make up-with-dev-auth`). |
| `OCTANEST_ADMIN_EMAIL` | Optional | _(unset)_ | With `OCTANEST_ADMIN_PASSWORD`, seeds one `sys-admin` on an **empty instance** (username `system-administrator`, `must_change_credentials` until `/setup/credentials`). Set both **before first boot**. If either/both unset and users is empty, the SPA `/setup` wizard creates the first `sys-admin` instead. Same path for cloud and self-host — no deployment-mode fork. Seed failure with both set → **fail boot** (exit 1). |
| `OCTANEST_ADMIN_PASSWORD` | Optional | _(unset)_ | Paired with `OCTANEST_ADMIN_EMAIL` for empty-instance `sys-admin` seed. Placeholders only in examples — never commit real passwords. |
| `OCTANEST_ALLOW_SIGNUP` | Optional | `false` | When `true`/`1`, ENV seed (and wizard default control) opens local signup (`allow_signup`). Default **false** (fail closed). After bootstrap, operators can change it via Admin → Auth. Cloud deploys that want open signup should set `true` in manifests. |
| `OCTANEST_RESEND_API_KEY` | Optional | _(unset)_ | Resend API key. Prefer this over SMTP when set (ENV boot selection). |
| `OCTANEST_RESEND_BASE_URL` | Optional | `https://api.resend.com` | Override Resend HTTP API base (local stubs). Leave unset in production. |
| `OCTANEST_SMTP_URL` | Optional | _(unset)_ | SMTP URL for lettre (e.g. `smtp://127.0.0.1:1025`). Used when Resend key is unset / provider is smtp. |
| `OCTANEST_MAIL_FROM` | Optional | `Octanest <noreply@localhost>` | Default From address when DB settings do not override. |
| `WORKOS_API_KEY` | Optional† | _(unset)_ | WorkOS API key (enterprise SSO). Required for WorkOS provider mode. |
| `WORKOS_CLIENT_ID` | Optional† | _(unset)_ | WorkOS AuthKit client id. |
| `OCTANEST_WORKOS_BASE_URL` | Optional | WorkOS SDK cloud default | Server-side WorkOS API base override for local stubs. Leave unset in production. |
| `OCTANEST_WORKOS_AUTHORIZE_BASE_URL` | Optional | Same as `OCTANEST_WORKOS_BASE_URL` / cloud | Browser-facing authorize base when it differs from the API base (e.g. Compose networking). |
| `OCTANEST_OIDC_ISSUER` | Optional† | _(unset)_ | OIDC issuer URL. Required for OIDC provider mode. |
| `OCTANEST_OIDC_CLIENT_ID` | Optional† | _(unset)_ | OIDC client id. |
| `OCTANEST_OIDC_CLIENT_SECRET` | Optional† | _(unset)_ | OIDC client secret. |
| `OCTANEST_OIDC_ALLOW_INSECURE` | Optional | unset / false | When `1`/`true`/`yes` **and** `OCTANEST_ENV` is `development`, `dev`, or `compose`, allows http/loopback issuers for local mocks. Never honored in production-like envs. |
| `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` | Compose | `octanest` | Postgres container credentials; must match `DATABASE_URL` for the default stack. |
| `MYSQL_USER` / `MYSQL_PASSWORD` / `MYSQL_DATABASE` / `MYSQL_ROOT_PASSWORD` | MySQL profile | `octanest` | MySQL container credentials (`docker-compose.mysql.yml`). |
| `MYSQL_DATABASE_URL` | MySQL profile | `mysql://octanest:octanest@mysql:3306/octanest` | Overrides API `DATABASE_URL` when using the MySQL Compose overlay. |
| `OCTANEST_SQLITE_HOST_DIR` | SQLite overlay | `./var` | Host path bind-mounted to `/app/var` for SQLite file storage. |

\* Strongly recommended for any real instance; without it the API runs with a skipped DB pool.  
† Required only when the corresponding auth provider mode is enabled (Admin → Auth / ENV bootstrap).

Dev-auth Compose port overrides (see `docker-compose.dev-auth.yml`): `OCTANEST_MAILPIT_SMTP_PORT` (1025), `OCTANEST_MAILPIT_UI_PORT` (8025), `OCTANEST_OIDC_MOCK_PORT` (9090), `OCTANEST_STUBS_PORT` (9092).

## Config file format

There is no separate application `config.json` / TOML. Configuration is:

1. **Shell / `.env`** — variables consumed by `octanest-api` and Compose `${…}` interpolation.
2. **Docker Compose YAML** — service wiring and dialect overlays:
   - `docker-compose.yml` — Traefik + web + api + Postgres
   - `docker-compose.mysql.yml` — MySQL profile
   - `docker-compose.sqlite.yml` — SQLite file bind-mount (no DB container)
   - `docker-compose.dev-auth.yml` — Mailpit, OIDC mock, Resend/WorkOS HTTP stubs

Minimal local `.env` for default Compose:

```bash
DATABASE_URL=postgres://octanest:octanest@postgres:5432/octanest
POSTGRES_USER=octanest
POSTGRES_PASSWORD=octanest
POSTGRES_DB=octanest
OCTANEST_AUTO_MIGRATE=true
OCTANEST_ENV=compose
OCTANEST_CORS_ORIGINS=http://localhost,http://127.0.0.1
API_BIND=0.0.0.0:8080
OCTANEST_RPC_VERSION=1
```

Auth settings that are **not** secrets (provider mode, from-address preference) are stored in the database and edited via Admin → Auth (`sys-admin` only). Secrets stay ENV-only.

### Recovering a DB with users but no `sys-admin`

ENV seed only runs when `users` is empty. If you already signed up during setup and need Auth settings access:

```sql
UPDATE users SET role = 'sys-admin' WHERE email = 'your@email';
```

Or wipe `users` / `sessions` and re-bootstrap with `OCTANEST_ADMIN_*` or the `/setup` wizard (empty DB only). Do **not** auto-promote on upgrade.

## Required vs optional settings

**Fail boot (exit 1):**

| Condition | Error / behavior |
|-----------|------------------|
| `OCTANEST_ENV` ∉ `{development,dev}` and `OCTANEST_CORS_ORIGINS` missing/empty | `cors config error: OCTANEST_CORS_ORIGINS is required when OCTANEST_ENV is not development` |
| Invalid origin string in CORS list | `invalid CORS origin …` |
| `DATABASE_URL` set with unknown scheme | `unrecognized DATABASE_URL scheme: …` |
| `OCTANEST_DB_DIALECT` disagrees with URL | `OCTANEST_DB_DIALECT=… does not match DATABASE_URL scheme (detected …)` |
| DB connect / migrate failure | Printed to stderr; process exits |
| Both `OCTANEST_ADMIN_*` set and empty-instance seed returns error | Fail closed: printed to stderr; process exits (no wizard fallback) |

**Do not fail boot if unset:** `DATABASE_URL` (warns and skips pool), email/SSO secrets (features degrade to log sink / unavailable provider), admin seed vars (wizard path when either/both unset on empty instance), `OCTANEST_ALLOW_SIGNUP` (defaults false).

## Defaults

| Setting | Default | Where |
|---------|---------|--------|
| `OCTANEST_ENV` | `development` | `crates/octanest-api/src/main.rs` |
| `API_BIND` | `0.0.0.0:8080` | `main.rs` |
| `OCTANEST_AUTO_MIGRATE` | `true` (any value other than `true`/`1` disables) | `main.rs` |
| `OCTANEST_ALLOW_SIGNUP` | `false` (only `true`/`1` opens signup at seed) | `crates/octanest-api/src/auth/seed.rs` / bootstrap |
| `OCTANEST_MAIL_FROM` | `Octanest <noreply@localhost>` | `crates/octanest-api/src/email/mod.rs` |
| Resend API base | `https://api.resend.com` | `crates/octanest-api/src/email/resend.rs` |
| Compose `DATABASE_URL` | `postgres://octanest:octanest@postgres:5432/octanest` | `docker-compose.yml` |
| Compose `OCTANEST_ENV` | `compose` | `docker-compose.yml` |
| Compose CORS | `http://localhost,http://127.0.0.1` | `docker-compose.yml` |
| Dialect | Inferred from `DATABASE_URL` scheme | `crates/octanest-db/src/dialect.rs` |

Email sender selection when building from ENV: Resend key → SMTP URL → log sink.

## Per-environment overrides

| Environment | Typical `OCTANEST_ENV` | CORS | Session cookie `Secure` | Stub / insecure flags |
|-------------|------------------------|------|-------------------------|------------------------|
| Host `make` + Vite (`:3000`) | `development` or `dev` | Mirror request origin; list optional | Off | `OCTANEST_*_BASE_URL` stubs + `OCTANEST_OIDC_ALLOW_INSECURE=1` allowed |
| Default Compose (Traefik `:80`) | `compose` | Allowlist required (`http://localhost`, …) | On (`compose` ≠ `development`/`dev`) | OIDC insecure flag allowed; use stub base URLs only for local stub stacks |
| Production-like | `production` (or other non-dev) | Allowlist required | On | Do **not** set stub base URLs or `OCTANEST_OIDC_ALLOW_INSECURE` |

**DATABASE_URL dialects by bring-up:**

| Dialect | Sample URL | Bring-up |
|---------|------------|----------|
| PostgreSQL | `postgres://octanest:octanest@postgres:5432/octanest` | `make up` |
| MySQL | `mysql://octanest:octanest@mysql:3306/octanest` | `make up-mysql` |
| SQLite | `sqlite:./var/octanest.db` (Compose: `sqlite:/app/var/octanest.db`) | `make up-sqlite` |

Host-local Postgres (API outside Compose): point `DATABASE_URL` at `localhost:5432` and set `OCTANEST_CORS_ORIGINS=http://localhost:3000` with `OCTANEST_ENV=development`.

**Local stubs without cloud secrets:** copy `docs/dev-auth.env.example` → `.env.dev-auth`, run `make up-dev-auth`, then `source` the file before starting the API. That sets SMTP/Mailpit, OIDC mock issuer, WorkOS/Resend stub bases (`http://127.0.0.1:9092`), and `OCTANEST_OIDC_ALLOW_INSECURE=1`. Details: [dev-auth.md](dev-auth.md).

<!-- VERIFY: Production WorkOS cloud API hostname when OCTANEST_WORKOS_BASE_URL is unset (SDK default; tests mention api.workos.com) -->
<!-- VERIFY: Deployed public origin / SSO redirect URIs for non-local environments -->
<!-- VERIFY: Production SMTP / Resend / WorkOS / OIDC secret values (ENV-only; not in repo) -->
