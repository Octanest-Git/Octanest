# Local auth & email stubs (no cloud secrets)

Run Mailpit + an OIDC mock + thin Resend/WorkOS HTTP stubs so you can exercise
SMTP, Resend, WorkOS, and OIDC paths without Dashboards or API keys.

## Quick start (recommended)

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

| Service | URL / port | Purpose |
|---------|------------|---------|
| Mailpit UI | http://127.0.0.1:8025 | Capture SMTP mail |
| Mailpit SMTP | `smtp://127.0.0.1:1025` | `OCTANEST_SMTP_URL` |
| OIDC mock | http://127.0.0.1:9090/default | Issuer for `OCTANEST_OIDC_*` |
| Stubs | http://127.0.0.1:9092 | Resend `POST /emails` + WorkOS AuthKit |

Tear down: `make down-dev-auth`.

## Automated stack e2e

True end-to-end coverage (real `octanest-api` + Mailpit + OIDC mock + HTTP stubs + Vite):

```bash
make test-e2e-stack
```

This boots the stub stack, SQLite API on `:18080`, Vite on `:13000`, then runs Vitest projects:

| Project | What it proves |
|---------|----------------|
| `e2e-stack` | SMTP→Mailpit, Resend→stub, WorkOS AuthKit stub login, OIDC mock login (HTTP) |
| `e2e-stack-browser` | Signup UI + WorkOS CTA against live web/API (Chromium) |

CI runs the same via the `e2e-stack` job. Default `bun run test` / `web-octane` stays fast (unit + integration + component e2e only).

## SMTP (Mailpit)

1. Keep `OCTANEST_SMTP_URL=smtp://127.0.0.1:1025` and leave `OCTANEST_RESEND_API_KEY` unset.
2. Sign up (local provider mode) and open Mailpit — welcome mail should appear.
3. In **Admin → Auth**, set email provider to **smtp** if you previously selected resend/log (boot already prefers ENV SMTP when Resend is unset).

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

1. Set provider mode to **oidc**.
2. Requires `OCTANEST_OIDC_ALLOW_INSECURE=1` and `OCTANEST_ENV` in `development` / `dev` / `compose` (never honored in production-like envs).
3. Issuer `http://127.0.0.1:9090/default` — any client id/secret accepted by the mock server.
4. Use Sign in with OIDC; the mock issues tokens without a real IdP account.

## Production safety

| Variable | Effect |
|----------|--------|
| `OCTANEST_RESEND_BASE_URL` | Points Resend HTTP client at a stub (unset in prod) |
| `OCTANEST_WORKOS_BASE_URL` | Points WorkOS SDK at a stub (unset in prod) |
| `OCTANEST_OIDC_ALLOW_INSECURE` | Allows http/loopback issuers only when `OCTANEST_ENV` is development/dev/compose |

Live WorkOS / Resend / SMTP credentials remain documented in [`.planning/phases/04-auth-sessions-email/04-USER-SETUP.md`](../.planning/phases/04-auth-sessions-email/04-USER-SETUP.md).
