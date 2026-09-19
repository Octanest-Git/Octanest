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

## Environments

| Environment | Role | Deploy trigger |
|-------------|------|----------------|
| `preview` | Persistent base for Railway **PR Environments** | Autodeploy off (IaC / manual only) |
| `staging` | Always-on integration | Autodeploy from `main` + Wait for CI |
| `production` | Live | Autodeploy off; promote via GitHub Action |

Ephemeral PR environments clone `preview` (services, networking, variables) when a project member opens a PR. They are deleted when the PR merges or closes. Bot PR Environments stay off unless you explicitly enable them.

**Collaborators:** invite people to the Railway **project** (Viewer is enough) and have them link their GitHub account. Repo collaborator status alone does not grant PR Environments.

**Focused PR Environments (recommended):** enable in Project Settings → Environments, then set watch paths on services, for example:

| Service | Watch paths |
|---------|-------------|
| `api` | `crates/octanest-api/**`, `crates/octanest-core/**`, `crates/octanest-db/**`, `packages/api-client/**` |
| `web` | `apps/web/**`, `packages/**` |
| `gateway` | `deploy/cloud/**` |

**Volume size:** `forge-data` is **20480 MB** (20 GB) in IaC.

## Operator workflow (D-CLOUD-07)

1. Install CLI ≥ 5.42.1 and link a project: `railway link`
2. From repo root: `cd .railway && npm ci` (or `npm install`)
3. Link the target environment, then preview: `make cloud-plan` (or `railway config plan`)
4. **Apply only with explicit human approval** — never from fork PR CI; never commit `RAILWAY_TOKEN`
5. Apply the same topology to `preview`, `staging`, and `production` (link each environment before plan/apply)

```bash
railway link --project <project-id> --environment preview
railway config plan
# review the plan, then only if approved:
railway config apply
```

Secrets (`OCTANEST_ENV`, `OCTANEST_PUBLIC_ORIGIN`, `OCTANEST_CORS_ORIGINS`, `OCTANEST_VITE_ALLOWED_HOSTS`, SSO/email keys, etc.) stay in the Railway dashboard or `preserve()` — not in git.

On **`web`**, set `OCTANEST_VITE_ALLOWED_HOSTS` so `vite preview` accepts the gateway Host header (e.g. `.up.railway.app,octanest.jereko.dev`). Details: [docs/CONFIGURATION.md](../docs/CONFIGURATION.md).

### Promote / rollback production

Do **not** rely on Environment Sync for promote: Sync includes variables and can clobber production-only origins and `OCTANEST_ENV`.

1. Confirm the commit is healthy on **staging**.
2. GitHub → **Actions** → **Production deploy** → Run workflow:
   - **promote** — `serviceInstanceDeployV2` with `commitSha` for `api` / `web` / `gateway` (default: `main` HEAD). Gated on CI success for that SHA. Leaves production variables alone.
   - **rollback** — `deploymentRollback` to the prior `canRollback` deployment on each of those services.
3. Smoke `https://octanest.jereko.dev/health`.

Script: [`scripts/railway-production-deploy.sh`](../scripts/railway-production-deploy.sh). Workflow: [`.github/workflows/production-deploy.yml`](../.github/workflows/production-deploy.yml).

**GitHub Environment `Octanest / production`:** add `RAILWAY_TOKEN`; enable required reviewers if you want an approval gate on the button.

## Related

- Gateway: [`deploy/cloud/README.md`](../deploy/cloud/README.md)
- Deploy steps: [`docs/DEPLOYMENT.md`](../docs/DEPLOYMENT.md)
