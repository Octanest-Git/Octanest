<!-- generated-by: gsd-doc-writer -->
# API

Octanest exposes a versioned JSON RPC over HTTP and WebSocket, plus a small set of browser-oriented auth and avatar routes. The Rust Axum router lives in `crates/octanest-api`; shared envelopes and DTOs live in `crates/octanest-core`. Clients should prefer the generated TypeScript package `@octanest/api-client`.

<!-- VERIFY: production / public base URL for the API -->

Local defaults: API bind `127.0.0.1:8080` (`API_BIND`), or same-origin via Traefik on `:80` (`/api`, `/uploads`, `/health`).

## Authentication

Session auth uses an **opaque HttpOnly cookie** named `octanest_session` (not JWTs or API keys).

| Detail | Value |
| --- | --- |
| Cookie name | `octanest_session` |
| Attributes | `HttpOnly`, `Path=/`, `SameSite=Lax`; `Secure` unless `OCTANEST_ENV` is `development` or `dev` |
| Idle TTL | 24 hours (sliding on resolve for non-remember sessions) |
| Remember-me TTL | 30 days absolute (`auth.login` with `remember_me: true`) |
| Storage | CSPRNG token in cookie; only SHA-256(token) stored in the DB |

**How to send credentials**

- Browser / generated client: `credentials: "include"` so the cookie is sent same-origin (Vite proxy or Traefik).
- Manual HTTP: include `Cookie: octanest_session=<token>`.
- Signup, login, and WorkOS/OIDC callbacks attach `Set-Cookie`. Logout / logout-all clear the cookie (`Max-Age=0`).

**Provider modes** (instance setting via `admin.auth.*`): `local` | `workos` | `oidc`. Local signup/login RPC only works when mode is `local`. SSO browser flows require matching mode and ENV secrets (see [CONFIGURATION.md](CONFIGURATION.md)).

**RPC version gate** — every `/api/rpc` and `/api/rpc/ws` request must send:

```http
Octanest-RPC-Version: 1
```

Missing or mismatched value → error `rpc.version_mismatch` (HTTP 400).

## Endpoints overview

### HTTP routes

| Method | Path | Description | Auth required |
| --- | --- | --- | --- |
| `GET` | `/health` | Liveness: `{"ok":true}` | No |
| `POST` | `/api/rpc` | JSON RPC dispatch | Cookie when procedure needs session |
| `GET` | `/api/rpc/ws` | WebSocket upgrade; same procedures as HTTP | Cookie when procedure needs session |
| `GET` | `/api/auth/workos/start` | Start WorkOS AuthKit (optional `?return_to=`) | No (redirect) |
| `GET` | `/api/auth/workos/callback` | WorkOS code exchange; sets session cookie | No (redirect) |
| `GET` | `/api/auth/oidc/start` | Start OIDC + PKCE (optional `?return_to=`) | No (redirect) |
| `GET` | `/api/auth/oidc/callback` | OIDC code exchange; sets session cookie | No (redirect) |
| `POST` | `/api/user/avatar` | Multipart avatar upload (field `avatar`) | Yes (`octanest_session`) |
| `GET` | `/uploads/avatars/{file}` | Public WebP avatar bytes (`{user_id}.webp`) | No |

SSO start routes redirect to the IdP when configured. If WorkOS/OIDC ENV is missing, start returns HTTP 503 with `auth.not_configured`. Failures typically redirect to `/login?error=sso`.

### RPC procedures (`POST /api/rpc`)

| Procedure | Description | Auth |
| --- | --- | --- |
| `system.health` | Status, API crate version, DB ping string | No |
| `system.echo` | Echo `message` (max 8192 bytes) | No |
| `system.db_probe` | Dialect probe / `instances` counter | No |
| `auth.signup` | Local signup; sets session cookie | No (local mode) |
| `auth.login` | Local login; sets session cookie | No (local mode) |
| `auth.logout` | Revoke current session; clear cookie | Session |
| `auth.logout_all` | Revoke all sessions for user; clear cookie | Session |
| `auth.me` | Current user public profile | Session |
| `auth.provider_config` | Public `{ mode }` for UI | No |
| `user.get_profile` | Current user profile | Session |
| `user.update_profile` | Update `display_name`, `username`, `bio` | Session |
| `admin.auth.get_settings` | Auth/email settings (no secrets) | Admin session |
| `admin.auth.update_settings` | Update provider/email settings; rebuild email sender | Admin session |

Unknown procedure → `rpc.unknown_procedure` (HTTP 404).

## Request/response formats

### RPC envelope

Request:

```json
{
  "procedure": "system.health",
  "input": {}
}
```

`input` defaults to `{}` if omitted. Success:

```json
{
  "ok": true,
  "data": {
    "status": "ok",
    "version": "…",
    "database": "ok"
  }
}
```

Failure:

```json
{
  "ok": false,
  "error": {
    "code": "auth.unauthenticated",
    "message": "not authenticated"
  }
}
```

Optional `error.data` may appear on some errors.

### Version header + curl example

```bash
curl -sS http://127.0.0.1:8080/api/rpc \
  -H 'content-type: application/json' \
  -H 'Octanest-RPC-Version: 1' \
  -d '{"procedure":"system.health","input":{}}'
```

Authenticated call (after login/signup returned `Set-Cookie`):

```bash
curl -sS http://127.0.0.1:8080/api/rpc \
  -H 'content-type: application/json' \
  -H 'Octanest-RPC-Version: 1' \
  -H 'Cookie: octanest_session=…' \
  -d '{"procedure":"auth.me","input":{}}'
```

### WebSocket `/api/rpc/ws`

1. Upgrade with `Octanest-RPC-Version: 1` (and optional `Cookie` for session).
2. Send text frames: same JSON as HTTP `RpcRequest`.
3. Receive text frames: same JSON as `RpcResponse`.
4. Invalid JSON frame → `rpc.bad_input`. Cookie `Set-Cookie` is HTTP-only; WS handlers do not attach cookies on responses.

### Local auth inputs

`auth.signup`:

```json
{ "email": "user@example.com", "username": "alice", "password": "at-least-8-chars" }
```

`auth.login`:

```json
{ "identifier": "alice", "password": "…", "remember_me": false }
```

`identifier` is email or username. Password minimum length: **8**. Response data is `UserPublic` (`id`, `email`, `username`, `display_name`, `bio`, `avatar_url`, `is_admin`, `profile_incomplete`).

### Profile

`user.update_profile`:

```json
{ "display_name": "Alice", "username": "alice", "bio": "" }
```

Bio max 160 characters; display name 1–100 characters. Avatar is **not** set via RPC — use multipart upload.

### Avatar upload

```bash
curl -sS http://127.0.0.1:8080/api/user/avatar \
  -H 'Cookie: octanest_session=…' \
  -F 'avatar=@photo.png;type=image/png'
```

- Field name: `avatar`
- Allowed types: `image/jpeg`, `image/png`, `image/webp`
- Max body: **2 MiB**
- Stored as WebP under `var/uploads/avatars/{user_id}.webp`; public URL `/uploads/avatars/{user_id}.webp`
- Success: `{"ok":true,"avatar_url":"/uploads/avatars/….webp"}`

### Admin auth settings

`admin.auth.update_settings` input (non-secret fields only; secrets stay in ENV):

```json
{
  "provider_mode": "local",
  "email_provider": "log",
  "from_address": "Octanest <noreply@example.com>",
  "oidc_issuer": null,
  "oidc_client_id": null,
  "workos_client_id": null
}
```

Response includes boolean badges such as `smtp_configured`, `resend_configured`, `workos_api_key_configured`, `oidc_client_secret_configured` — never raw secrets.

### TypeScript client

```ts
import { createClient } from "@octanest/api-client";

const client = createClient({ baseUrl: "" }); // same-origin; credentials: "include" by default
const health = await client.system.health();
const me = await client.auth.me();
```

TanStack Query helpers (`authMeQueryOptions`, `adminAuthGetSettingsQueryOptions`, etc.) are exported from the same package.

## Error codes

HTTP status for `/api/rpc` is derived from the RPC error:

| HTTP | When |
| --- | --- |
| `200` | `ok: true` |
| `400` | Most RPC errors (validation, provider mismatch, version mismatch, etc.) |
| `401` | `auth.unauthenticated` |
| `403` | `admin.forbidden` |
| `404` | `rpc.unknown_procedure` |

Common `error.code` values:

| Code | Meaning |
| --- | --- |
| `rpc.version_mismatch` | Missing/wrong `Octanest-RPC-Version` |
| `rpc.bad_input` | Invalid JSON / procedure input |
| `rpc.payload_too_large` | Echo message too large |
| `rpc.unknown_procedure` | Unknown procedure name |
| `auth.unauthenticated` | No valid session |
| `auth.provider_mismatch` | Local auth disabled for current mode |
| `auth.taken` / `auth.invalid_*` / `auth.weak_password` / `auth.reserved_username` | Signup/profile validation |
| `auth.not_configured` | WorkOS/OIDC ENV missing (SSO start) |
| `admin.forbidden` | Authenticated but not admin |
| `db.not_configured` / `db.probe_failed` | Database unavailable |
| `avatar.*` | Multipart/type/size/store failures on avatar upload |

Avatar and SSO JSON errors use the same `{ ok: false, error: { code, message } }` shape where applicable.

## Rate limits

No application-level rate limiting is configured in `octanest-api` (no rate-limit middleware or dependency detected). Rely on reverse-proxy / edge controls if needed in deployment.

## Regenerating the TypeScript client

Procedure names and DTOs in Rust (`rpc.rs`, `octanest-core`) are authoritative. Regenerate `@octanest/api-client`:

```bash
make rpc-gen
# equivalent: cargo run -q -p octanest-api --bin rpc-gen
```

This overwrites `packages/api-client/src/index.ts` (do not hand-edit). CI drift check:

```bash
make rpc-sync-check
```

Related docs: [ARCHITECTURE.md](ARCHITECTURE.md), [CONFIGURATION.md](CONFIGURATION.md), [database.md](database.md), [dev-auth.md](dev-auth.md).
