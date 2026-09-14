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

**Personal access tokens (PATs) are not RPC Bearer credentials (D-01).** Typed `/api/rpc` and `/api/rpc/ws` use the session cookie only. PATs authenticate **Git Smart HTTP** over HTTPS via HTTP Basic (password = token). Do not send `Authorization: Bearer <pat>` to RPC — it is ignored for session resolution.

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
| `GET` | `/{owner}/{repo}.git/info/refs` | Git Smart HTTP discovery (`?service=git-upload-pack` \| `git-receive-pack`) | PAT Basic when required (not session) |
| `POST` | `/{owner}/{repo}.git/git-upload-pack` | Git fetch / clone body | PAT Basic when required (not session) |
| `POST` | `/{owner}/{repo}.git/git-receive-pack` | Git push body | PAT Basic (verified email; write scope) |
| `POST` | `/{owner}/{repo}.git/info/lfs/objects/batch` | Git LFS Batch API | PAT Basic (Read download / Write upload) |
| `PUT` | `/{owner}/{repo}.git/info/lfs/objects/{oid}` | LFS basic transfer upload (streaming) | PAT Basic (Write) |
| `GET` | `/{owner}/{repo}.git/info/lfs/objects/{oid}` | LFS basic transfer download (Range supported) | PAT Basic (Read) |
| `POST` | `/{owner}/{repo}.git/info/lfs/objects/{oid}/verify` | Optional LFS verify | PAT Basic (Write) |

SSO start routes redirect to the IdP when configured. If WorkOS/OIDC ENV is missing, start returns HTTP 503 with `auth.not_configured`. Failures typically redirect to `/login?error=sso`.

### RPC procedures (`POST /api/rpc`)

| Procedure | Description | Auth |
| --- | --- | --- |
| `system.health` | Status, API crate version, DB ping string | No |
| `system.echo` | Echo `message` (max 8192 bytes) | No |
| `system.db_probe` | Dialect probe / `instances` counter | No |
| `auth.signup` | Local signup; sets session cookie. Rejected with `auth.setup_required` while empty-instance setup is needed; rejected when instance `allow_signup` is false | No (local mode) |
| `auth.login` | Local login; sets session cookie. ENV-seeded admins with `must_change_credentials` are redirected to `/setup/credentials` in the SPA | No (local mode) |
| `auth.logout` | Revoke current session; clear cookie | Session |
| `auth.logout_all` | Revoke all sessions for user; clear cookie | Session |
| `auth.me` | Current user public profile (`must_change_credentials` included) | Session |
| `auth.provider_config` | Public `{ mode, allow_signup }` for UI (fail-closed when unset/error) | No (blocked while `needs_setup`) |
| `auth.bootstrap_status` | `{ needs_setup }` — empty users table and incomplete `OCTANEST_ADMIN_*` ENV | No |
| `auth.bootstrap_setup` | One-time `/setup` wizard; creates verified `sys-admin` + session; persists `allow_signup` | No (empty instance only) |
| `auth.confirm_admin_credentials` | Forced credential change for ENV-seeded admins (`system-administrator` must be changed; email/password may be kept) | Session (seeded admin) |
| `user.get_profile` | Current user profile | Session |
| `user.update_profile` | Update `display_name`, `username`, `bio` | Session |
| `user.lookup` | Username prefix autocomplete (public fields only; never emails) | Session (rate-limited) |
| `org.create` | Create organization; slug shares username reserved list | Session + verified email |
| `org.get` | Org profile by slug | Session + verified |
| `org.listMine` | Orgs the caller belongs to (includes caller `role`) | Session + verified |
| `org.updateSettings` | Update `display_name` / `member_base_permission` | Org Admin+ |
| `org.members.list` | Members (`username`, `role`, ids — no emails) | Org member |
| `org.members.add` / `updateRole` / `remove` | Membership mutations | Org Admin+ (Owner-only for Owner grants) |
| `org.invites.create` / `list` / `revoke` | Email invites (plaintext token only in outbound mail link) | Org Admin+ |
| `org.invites.accept` | Redeem invite token; may create verified user under closed signup | Token (optional session) |
| `repo.listMine` / `repo.listByOwner` | Personal / owner-scoped repo lists (ACL-filtered) | Session |
| `repo.create` / `repo.get` / browse / branch / settings | Forge RPC (Capability ACL) | Session (+ capability) |
| `repo.collaborators.list` / `add` / `update` / `remove` | Per-repo collaborator grants | Repo Admin |
| `admin.auth.get_settings` | Auth/email settings including `allow_signup` (no secrets) | Admin session |
| `admin.auth.update_settings` | Update provider/email/`allow_signup`; rebuild email sender | Admin session |
| `admin.instance.factory_reset` | Wipe users, orgs, repos + issue domain (DB); optional disk wipe via `scope` | Sys-admin |
| `issue.create` / `get` / `list` / `update` / `close` / `reopen` / `history` / `delete` | Per-repo issues (`#N`); Capability ACL | Session (+ capability) |
| `issue.comments.*` | Comment CRUD + history; author or Write+ moderate-delete | Session (+ capability) |
| `issue.labels.set` / `assignees.set` / `assigneeCandidates` | Assign labels / assignees (Write+; assignees must have Read+) | Session (+ capability) |
| `issue.reactions.toggle` | Toggle GitHub-style reaction on issue or comment | Session (+ Write+) |
| `issue.links.list` / `add` / `remove` | Linked PR stubs + manual links (`pr_stub`) | Session (+ capability) |
| `label.listForRepo` / `listForOrg` / `create` / `update` / `delete` | Org/repo label definitions (Admin for defs) | Session (+ capability) |
| `pat.createClassic` | Mint classic PAT (`octanest_pat_…`); one-time plaintext in response | Session + verified email |
| `pat.createFineGrained` | Mint fine-grained PAT (`octanest_fg_…`); one-time plaintext in response | Session + verified email |
| `pat.list` | List active PATs for the signed-in user (no secrets) | Session |
| `pat.revoke` | Soft-revoke a PAT by `id` | Session |
| `sshKey.add` | Register an OpenSSH public key; returns fingerprint metadata | Session + verified email |
| `sshKey.list` | List registered SSH public keys (no private keys) | Session |
| `sshKey.revoke` | Hard-delete an SSH public key by `id` | Session |

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

`identifier` is email or username. Password minimum length: **8**. Response data is `UserPublic` (`id`, `email`, `username`, `display_name`, `bio`, `avatar_url`, `role` (`user` \| `admin` \| `sys-admin`), `profile_incomplete`, `email_verified`).

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

### Personal access tokens (`pat.*`)

Manage tokens with the session cookie via RPC (or `@octanest/api-client`). Token **prefixes** (redacted examples only):

| Kind | Prefix | Phase 8 capability |
| --- | --- | --- |
| Classic | `octanest_pat_` | Scope catalog: `repo` (HTTPS fetch + push where ACL allows) |
| Fine-grained | `octanest_fg_` | `repo_access`: `selected` \| `all`; `contents`: `read` \| `write` |

`pat.createClassic` input:

```json
{ "name": "laptop", "scopes": ["repo"], "expires_at": null }
```

`pat.createFineGrained` input:

```json
{
  "name": "ci-bot",
  "repo_access": "selected",
  "repository_ids": ["…repo-id…"],
  "contents": "write",
  "expires_at": null
}
```

Create responses include a one-time plaintext `token` (store it immediately) plus a metadata `item` **without** the secret. `pat.list` / list items never return the secret — only `token_prefix`, scopes/permissions, `last_used_at` / `last_used_ip`, etc. `pat.revoke` input: `{ "id": "…" }`.

Minting requires a verified email (`auth.email_unverified` otherwise). Empty note/name → `pat.note_required`. Selected fine-grained with no repositories → `pat.repos_required`. Invalid classic scopes or foreign/empty-id fine-grained `selected` repos → `pat.invalid_scope`. Unknown or non-owned revoke id → `pat.not_found`.

### SSH public keys (`sshKey.*`)

Register OpenSSH **public** keys for Git-over-SSH (session cookie; never send private keys to the API). Keys map to **full account identity** — there are no PAT-style scopes on SSH transport (D-SSH-03 / D-SSH-04 / D-SSH-05).

`sshKey.add` input:

```json
{ "title": "laptop", "public_key": "ssh-ed25519 AAAA… comment" }
```

Accepted key types: `ssh-ed25519` and RSA ≥2048. Response is a list item with `id`, `title`, `fingerprint` (SHA256), `key_type`, optional `public_key`, `created_at`, and optional `last_used_*`. There is **no** one-time secret field (unlike PAT mint).

`sshKey.list` returns the same item shape for the signed-in user. `sshKey.revoke` input: `{ "id": "…" }` (hard-delete).

Add requires verified email (`auth.email_unverified` otherwise). Empty title → `sshKey.title_required`. Invalid/unsupported key → `sshKey.invalid_key`. Duplicate fingerprint → `sshKey.fingerprint_taken`. More than **25** keys → `sshKey.limit_exceeded`. Unknown or non-owned revoke id → `sshKey.not_found`.


**PAT ∩ ACL:** Classic `repo` push/fetch requires the PAT subject to also `meets` the needed Capability on that repository (org membership, collaborator grant, or personal owner — not `owner_id == pat.user_id` alone). Fine-grained `all` covers personal-owned plus org Owner/Admin repos; collaborators must use `selected`.

### Organizations (`org.*`) & collaborators

Organizations share the username slug namespace. `org.create` rejects reserved / taken slugs (`org.slug_taken`, `auth.reserved_username`). Blank `display_name` defaults to the slug. `member_base_permission` defaults to `none` and applies only to org **Members** on org-owned private repos (Owner/Admin always Admin).

`org.members.list` returns username + role + ids only (no emails). Live add uses `user.lookup` (prefix ≥ 2; short/email-shaped prefixes return empty ok). Invite create/list omit plaintext tokens; accept redeems the magic-link token and can create a verified local user even when instance `allow_signup` is false. If the invite email already has an account, accept returns `org.invite_login_required` instead of overwriting credentials.

`repo.collaborators.*` grants per-repo `read` \| `write` \| `admin` (never an org role). Mutations require repo Admin capability. Highest-wins coalesce with org roles / `member_base` (collaborator raises effective permission; cannot lower Owner/Admin).

`admin.instance.factory_reset` (`confirmation: "RESET"`) wipes repositories (cascades collaborators, PAT-repo links, and **issue domain** tables), organizations (members/invites/org-scoped labels cascade), and auth users. `scope`: `database_only` (default) keeps bare dirs; `database_and_repositories` also clears `OCTANEST_REPOS_DIR` children.

### Issues (`issue.*`) & labels (`label.*`)

Phase 11 ships per-repository issues (ISS-01…04) on migration `0011_issues`:

| Concern | Contract |
| --- | --- |
| **Numbering** | Each repo allocates monotonic `#N` via `issue_counters`. Hard-delete does **not** reclaim numbers. |
| **ACL** | Capability gates: Read+ to view; Write+ to create/comment/assign/react/link; author or Write+ to edit own issue/comment; Admin (or typed confirm) for hard-delete. Private unauthorized access returns soft `repo.not_found` / `issue.not_found` (no enumeration). |
| **Markdown** | Web Write\|Preview uses `renderGfm` with `#N` / `owner/repo#N` autolink and sanitize-last. `@mention` / commit SHA autolink are off. |
| **Linked PRs** | `issue.links.*` stores stub rows (`pr_stub`) until Phase 12 PR objects exist. Manual add/remove only. |
| **Deferred** | Closing keywords (`fixes` / `closes` `#N`) are **not** enforced (D-ISS-15 → Phase 12). |

Client surface: `client.issue.*` / `client.label.*` in `@octanest/api-client` (regenerate with `make rpc-gen`).

### Git Smart HTTP

Clone / fetch / push use Git Smart HTTP under `/{owner}/{repo}.git` (not `/api/rpc`):

| Method | Path | Service |
| --- | --- | --- |
| `GET` | `/{owner}/{repo}.git/info/refs?service=git-upload-pack` | Discovery (fetch) |
| `GET` | `/{owner}/{repo}.git/info/refs?service=git-receive-pack` | Discovery (push) |
| `POST` | `/{owner}/{repo}.git/git-upload-pack` | Fetch / clone |
| `POST` | `/{owner}/{repo}.git/git-receive-pack` | Push |

**Auth:** HTTP Basic with password = PAT (`octanest_pat_…` or `octanest_fg_…`). Username may be the account username or aliases `git`, `token`, or `oauth2` (identity comes from the PAT hash). Account passwords are rejected. **Session cookies are ignored** for Smart HTTP authorization.

Public repos may allow anonymous `upload-pack`. Private repos and push require a valid PAT with sufficient **scope ∩ Capability ACL**; ACL denials stay HTTP **401** Basic, while insufficient PAT scope → HTTP **403**. Unverified-email users may fetch but not push (`auth.email_unverified` JSON on receive-pack). Failed Basic auth may return `401` with `WWW-Authenticate: Basic realm="Octanest Git"` and a PAT hint body.

Example (redacted token):

```bash
git clone https://git:octanest_pat_REDACTED@example.com/alice/demo.git
# or:
git -c http.extraHeader="Authorization: Basic $(printf 'git:octanest_pat_REDACTED' | base64 -w0)" \
  ls-remote https://example.com/alice/demo.git
```

### Git LFS

Phase 14 serves **Git LFS** over HTTPS under the same `{owner}/{repo}.git` surface (Traefik `.git` PathRegexp already covers `info/lfs`). Storage is the instance volume `OCTANEST_LFS_DIR` — see [CONFIGURATION.md](CONFIGURATION.md#git-lfs).

| Method | Path | Role |
| --- | --- | --- |
| `POST` | `/{owner}/{repo}.git/info/lfs/objects/batch` | Batch discover upload/download actions (`transfer=basic`) |
| `PUT` | `/{owner}/{repo}.git/info/lfs/objects/{oid}` | Streaming basic upload |
| `GET` | `/{owner}/{repo}.git/info/lfs/objects/{oid}` | Download; optional `Range` |
| `POST` | `/{owner}/{repo}.git/info/lfs/objects/{oid}/verify` | Optional post-upload verify |

**Auth (D-LFS-09):** Same as Smart HTTP — HTTP Basic with password = **personal access token**. Username aliases `git` / `token` / `oauth2` work. **Session cookies are ignored** for LFS. Failed auth may return `401` with `WWW-Authenticate: Basic realm="Octanest Git"` (LFS clients also accept `LFS-Authenticate`). Read capability for download; Write + verified email for upload. Classic `repo` / fine-grained `contents` scopes (no dedicated `lfs` scope).

**Enable:** Per-repo LFS must be enabled by a repository Admin before Batch issues upload actions. Quotas / max object size reject oversized uploads with clear LFS error JSON (no soft-warn-only).

**Client setup:** Track patterns with `git lfs track` and commit `.gitattributes` (server does not auto-commit). Example:

```bash
git lfs install
git lfs track "*.psd"
git add .gitattributes
# push with HTTPS remote + PAT, or SSH git remote + HTTPS LFS via credential helper
```

**SSH git remotes:** Pack protocol may use SSH, but LFS object transfer remains **HTTPS** in Phase 14. Configure a credential helper so `git-lfs` can present a PAT to `https://…/{owner}/{repo}.git/info/lfs`. LFS-over-SSH is not supported.

### Git over SSH

Clone / fetch / push over SSH use an in-process listener (Compose TCP **2222** by default — not Traefik). Remotes are **scp-style** `git@{host}:{owner}/{repo}.git` (D-SSH-02). The SSH username must be `git`; identity comes only from a registered public-key fingerprint (full account ACL — no PAT scopes). When advertised port ≠ 22, clients set `Port` in `~/.ssh/config` (or `ssh -p`); do not treat `ssh://` as the primary CloneBox URL.

Failed pubkey auth is rate-limited like Smart HTTP PAT failures (IP + fingerprint buckets). See [CONFIGURATION.md](CONFIGURATION.md) for `OCTANEST_SSH_*`.

### TypeScript client

```ts
import { createClient } from "@octanest/api-client";

const client = createClient({ baseUrl: "" }); // same-origin; credentials: "include" by default
const health = await client.system.health();
const me = await client.auth.me();
const pats = await client.pat.list();
const created = await client.pat.createClassic({ name: "laptop", scopes: ["repo"] });
// created.data.token is shown once — never send it as RPC Bearer
const keys = await client.sshKey.list();
await client.sshKey.add({ title: "laptop", public_key: "ssh-ed25519 AAAA… comment" });
```

TanStack Query helpers (`authMeQueryOptions`, `patListQueryOptions`, `sshKeyListQueryOptions`, `adminAuthGetSettingsQueryOptions`, etc.) are exported from the same package.

## Error codes

HTTP status for `/api/rpc` is derived from the RPC error:

| HTTP | When |
| --- | --- |
| `200` | `ok: true` |
| `400` | Most RPC errors (validation, provider mismatch, version mismatch, etc.) |
| `401` | `auth.unauthenticated` |
| `403` | `admin.forbidden`, `auth.email_unverified` |
| `404` | `rpc.unknown_procedure`, `repo.not_found`, `issue.not_found` |

Common `error.code` values:

| Code | Meaning |
| --- | --- |
| `rpc.version_mismatch` | Missing/wrong `Octanest-RPC-Version` |
| `rpc.bad_input` | Invalid JSON / procedure input |
| `rpc.payload_too_large` | Echo message too large |
| `rpc.unknown_procedure` | Unknown procedure name |
| `auth.unauthenticated` | No valid session |
| `auth.email_unverified` | Verified email required (PAT mint; SSH key add; Smart HTTP / SSH push) |
| `auth.provider_mismatch` | Local auth disabled for current mode |
| `auth.taken` / `auth.invalid_*` / `auth.weak_password` / `auth.reserved_username` | Signup/profile validation |
| `auth.setup_required` | Empty instance must complete `/setup` before signup/SSO |
| `auth.setup_unavailable` | `/setup` already completed (users exist or ENV seed path) |
| `auth.not_configured` | WorkOS/OIDC ENV missing (SSO start) |
| `admin.forbidden` | Authenticated but not admin |
| `pat.note_required` | PAT name/note empty |
| `pat.repos_required` | Fine-grained `selected` with no repository ids |
| `pat.invalid_scope` | Classic scopes or fine-grained repo selection invalid |
| `pat.not_found` | Revoke target missing or not owned |
| `sshKey.title_required` | SSH key title/note empty |
| `sshKey.invalid_key` | Public key parse/type/size rejected |
| `sshKey.fingerprint_taken` | Fingerprint already registered |
| `sshKey.limit_exceeded` | More than 25 SSH keys for the user |
| `sshKey.not_found` | Revoke target missing or not owned |
| `org.slug_taken` | Org slug collides with user or org |
| `org.forbidden` / `org.not_found` | Org ACL / missing org |
| `org.invite_login_required` | Invite email already registered — sign in to accept |
| `org.invite_*` | Invite expired / revoked / invalid |
| `repo.not_found` | Missing or unauthorized private (web/RPC soft 404) |
| `repo.create_forbidden` | Org Member cannot create under that org |
| `issue.not_found` / `issue.comment_not_found` / `issue.link_not_found` | Missing issue/comment/link (private soft-404 where applicable) |
| `issue.confirm_mismatch` | Admin hard-delete confirmation number mismatch |
| `label.not_found` | Missing label definition |
| `db.not_configured` / `db.probe_failed` | Database unavailable |
| `avatar.*` | Multipart/type/size/store failures on avatar upload |

Avatar and SSO JSON errors use the same `{ ok: false, error: { code, message } }` shape where applicable.

## Rate limits

Smart HTTP failed-authentication attempts are rate-limited in-process: **20 failures per client IP** and **10 per username** per **15 minutes**, then HTTP `429` with `Retry-After`. Client IP uses the rightmost `X-Forwarded-For` hop from a trusted proxy; do not expose the API without a proxy that sanitizes forwarded headers. Successful PAT auth clears the user bucket. Git-over-SSH failed pubkey auth uses the same windows with the key **fingerprint** as the user bucket. Other RPC routes do not apply this limiter; rely on reverse-proxy / edge controls for deployment-wide limits.

`user.lookup` is rate-limited per session (**60** requests / **60s**). Other RPC routes do not apply in-process limiters; rely on reverse-proxy / edge controls for deployment-wide limits.

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
