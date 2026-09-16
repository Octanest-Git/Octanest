# Octanest Cloud — Railway IaC

TypeScript Infrastructure as Code for **Octanest Cloud** on Railway (PLAT-02 / D-CLOUD-01…07).

## Layout

| File | Role |
|------|------|
| `railway.ts` | Desired topology: managed Postgres + `api` + `web` + `gateway` |
| `package.json` | Isolated npm env for the `railway` IaC SDK (not part of Bun workspaces) |

**Do not** add deprecated `railway.json` / `railway.toml` (cutoff 2026-12-01).

## Same images as Compose

| Service | Builder | Dockerfile |
|---------|---------|------------|
| `api` | `DOCKERFILE` | `crates/octanest-api/Dockerfile` (repo-root context) |
| `web` | `DOCKERFILE` | `apps/web/Dockerfile` (repo-root context) |
| `gateway` | `DOCKERFILE` | `deploy/cloud/Dockerfile` (Caddy file proxy) |

Cloud default database is **managed Postgres** (`postgres()` helper). MySQL/SQLite remain self-host/CI dialects only (D-CLOUD-02).

## Operator workflow (D-CLOUD-07)

1. Install CLI ≥ 5.42.1 and link a project: `railway link`
2. From repo root: `cd .railway && npm ci` (or `npm install`)
3. Preview: `make cloud-plan` (or `railway config plan`)
4. **Apply only with explicit human approval** — never from fork PR CI; never commit `RAILWAY_TOKEN`

```bash
railway config plan
# review the plan, then only if approved:
railway config apply
```

Secrets (`OCTANEST_CORS_ORIGINS`, SSO/email keys, etc.) stay in the Railway dashboard or `preserve()` — not in git.

## Related

- Gateway: [`deploy/cloud/README.md`](../deploy/cloud/README.md)
- Deploy steps: [`docs/DEPLOYMENT.md`](../docs/DEPLOYMENT.md)
