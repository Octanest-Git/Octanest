# Local auth & email stubs (no cloud secrets)

Run Mailpit + an OIDC mock + thin Resend/WorkOS HTTP stubs so you can exercise
SMTP, Resend, WorkOS, and OIDC paths without Dashboards or API keys.

## Quick start

### A — Host API + Vite (`make dev`)

```bash
cp docs/dev-auth.env.example .env.dev-auth
make up-dev-auth

# Load throwaway env into your shell, then start the normal host stack:
set -a && source .env.dev-auth && set +a
make rpc-gen
# terminal 1
OCTANEST_ENV=development API_BIND=127.0.0.1:8080 cargo run -p octanest-api --bin octanest-api
# terminal 2
bun run --filter @octanest/web dev
```

### B — Compose stack (`make up`) + stubs (SMTP → Mailpit)

Keep Traefik / web / API / Postgres from Compose, and point the **API container** at Mailpit over Docker DNS:

```bash
make up-with-dev-auth
```

Then open:

| Surface | URL |
|---------|-----|
| App | http://localhost |
| Mailpit UI | http://127.0.0.1:8025 |

Defaults set `OCTANEST_SMTP_URL=smtp://mailpit:1025` via `OCTANEST_COMPOSE_SMTP_URL` (so a host `.env` with `127.0.0.1:1025` for `make dev` does not break the API container). Magic links use `OCTANEST_COMPOSE_PUBLIC_ORIGIN` → `http://localhost` (Traefik), so a host `OCTANEST_PUBLIC_ORIGIN=http://localhost:3000` for Vite does not leak into mail. Optional Resend / WorkOS / OIDC stub vars: `docs/dev-auth.env.compose.example` (merge into `.env`).

Tear down stubs only: `make down-dev-auth`. Full stack from `up-with-dev-auth`: `make down-with-dev-auth`.

| Service | Host URL / port | Compose API URL |
|---------|-----------------|-----------------|
| Mailpit UI | http://127.0.0.1:8025 | — |
| Mailpit SMTP | `smtp://127.0.0.1:1025` | `smtp://mailpit:1025` |
| OIDC mock | http://127.0.0.1:9090/default | `http://host.docker.internal:9090/default` |
| Stubs | http://127.0.0.1:9092 | `http://stubs:9092` |

If `make up-dev-auth` fails with `docker-credential-desktop.exe: executable file not found` (common on WSL), re-run after pulling this Makefile fix, or temporarily:

```bash
DOCKER_CONFIG=$(mktemp -d) && echo '{}' >"$DOCKER_CONFIG/config.json" && export DOCKER_CONFIG && make up-dev-auth
```

`make up-dev-auth` / `make up-with-dev-auth` auto-fall back to a local empty Docker config when that helper is missing.

## Automated stack e2e

True end-to-end coverage (real `octanest-api` + Mailpit + OIDC mock + HTTP stubs + Vite):

```bash
make test-e2e-stack
```

This boots the stub stack, SQLite API on `:18080`, Vite on `:13000`, then runs Vitest projects:

| Project | What it proves |
|---------|----------------|
| `e2e-stack` | SMTP→Mailpit, Resend→stub, WorkOS AuthKit stub login, OIDC mock login (HTTP) |
| `e2e-stack-browser` | Signup UI, WorkOS CTA, OIDC SSO click-through (Chromium) |

CI runs the same via the `e2e-stack` job. Default `bun run test` / `web-octane` stays fast (unit + integration + component e2e only).

## SMTP (Mailpit)

**Host API (`make dev`):** keep `OCTANEST_SMTP_URL=smtp://127.0.0.1:1025` and leave `OCTANEST_RESEND_API_KEY` unset.

**Compose API (`make up-with-dev-auth`):** defaults to `smtp://mailpit:1025` via `OCTANEST_COMPOSE_SMTP_URL` — do not point the API container at `127.0.0.1`. The Makefile also promotes DB `email_provider` from `log` → `smtp` (seed default is log, which would otherwise keep mail in the API log sink).

Then:

1. Sign up or click **Resend email**, then open Mailpit — verify mail should appear.
2. Or set **Admin → Auth → email provider** to **smtp** if you previously chose log/resend.

## Resend stub

1. Comment out `OCTANEST_SMTP_URL`.
2. Set `OCTANEST_RESEND_API_KEY=re_dev_local` and `OCTANEST_RESEND_BASE_URL=http://127.0.0.1:9092`.
3. Set admin email provider to **resend** (or rely on boot ENV selection).
4. Stub logs accept any Bearer token and return `{ "id": "email_dev_local" }`.

## WorkOS stub

1. Set provider mode to **workos** in Admin → Auth (or empty users + ENV bootstrap).
2. With `WORKOS_*` + `OCTANEST_WORKOS_BASE_URL` from `.env.dev-auth`, Sign in → WorkOS redirects through the stub and lands back on `/api/auth/workos/callback` as `dev@octanest.local`.

`OCTANEST_WORKOS_AUTHORIZE_BASE_URL` is only needed when the browser-facing base differs from the API base (e.g. Compose networking).

## OIDC mock

1. Set provider mode to **oidc** (Admin → Auth, or choose OIDC during `/setup`).
2. Requires `OCTANEST_OIDC_ALLOW_INSECURE=1` and `OCTANEST_ENV` in `development` / `dev` / `compose` (never honored in production-like envs).
3. Host issuer `http://127.0.0.1:9090/default` — any client id/secret accepted by the mock server.
4. Compose API must use `OCTANEST_COMPOSE_OIDC_ISSUER=http://host.docker.internal:9090/default` (never `127.0.0.1` inside the API container). On Linux without Docker Desktop, ensure `host.docker.internal` resolves (often `extra_hosts` / `/etc/hosts`).
5. Use Sign in with SSO; the mock issues tokens without a real IdP account.
6. If discovery hangs, check the mock is up (`curl -4 -m 3 http://127.0.0.1:9090/default/.well-known/openid-configuration`) — the API now fails OIDC HTTP within ~10s instead of spinning forever.

## Production safety

| Variable | Effect |
|----------|--------|
| `OCTANEST_RESEND_BASE_URL` | Points Resend HTTP client at a stub (unset in prod) |
| `OCTANEST_WORKOS_BASE_URL` | Points WorkOS SDK at a stub (unset in prod) |
| `OCTANEST_OIDC_ALLOW_INSECURE` | Allows http/loopback issuers only when `OCTANEST_ENV` is development/dev/compose |

Live WorkOS / Resend / SMTP credentials remain documented in [`.planning/phases/04-auth-sessions-email/04-USER-SETUP.md`](../.planning/phases/04-auth-sessions-email/04-USER-SETUP.md).
