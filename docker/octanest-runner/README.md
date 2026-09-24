# Octanest official Actions runner

Runs the native `octanest-runner` binary (`crates/octanest-runner`) against
Octanest's JSON runner protocol (`/api/actions/{register,declare,fetch_task,update_task,update_log}`).
Operators supply compute; the API stays a control plane and never executes
workflow steps in-process.

**Published image:** `ghcr.io/<owner>/octanest-runner` via
`.github/workflows/publish-runner.yml` (main + semver tags).

## What the runner executes

- `run:` steps — host mode (default) or inside a container for
  `docker://` labels.
- `uses: actions/checkout[@ref]` — built-in; clones
  `{OCTANEST_PUBLIC_ORIGIN}/{owner}/{repo}.git` and detaches at the run's
  head sha.
- Any other `uses:` action fails the job loudly (not silently skipped).

Secrets from the task payload are injected as step env vars and masked out
of uploaded log chunks.

## Labels

`OCTANEST_RUNNER_LABELS` is a comma-separated list:

- `ubuntu-latest` → host execution (no Docker needed)
- `ubuntu-latest:host` → explicit host execution
- `ubuntu-latest:docker://node:20-bookworm` → steps run in `docker run` with
  that image (requires the Docker socket mount)

## Environment

| Var | Default | Purpose |
|-----|---------|---------|
| `OCTANEST_PUBLIC_ORIGIN` | — (required) | Base URL for the API **and** clone URLs. Must be reachable from the runner; for docker:// jobs the clone happens runner-side. |
| `OCTANEST_RUNNER_REGISTRATION_TOKEN` | — | One-shot bootstrap token. Required until first register persists state. Matches the API's `OCTANEST_RUNNER_REGISTRATION_TOKEN` env or a minted DB token. |
| `OCTANEST_RUNNER_NAME` | `$HOSTNAME` | Runner display name. |
| `OCTANEST_RUNNER_LABELS` | `ubuntu-latest,self-hosted` | See above. |
| `OCTANEST_RUNNER_STATE` | `/data/runner.json` | Persisted registration (runner id + bearer token). |
| `OCTANEST_RUNNER_WORK_DIR` | `/data/work` | Per-job workspaces (cleaned per job). |
| `OCTANEST_RUNNER_POLL_MS` | `2000` | fetch_task poll interval. |
| `OCTANEST_RUNNER_GIT_TOKEN` | — | PAT for cloning **private** repos (sent via `http.extraHeader`, never in URLs). |
| `OCTANEST_RUNNER_JOB_TIMEOUT_SECS` | `3600` | Per-step timeout. |

## Run

```bash
docker build -t octanest-runner -f docker/octanest-runner/Dockerfile .

docker run --rm -it \
  -e OCTANEST_PUBLIC_ORIGIN=https://git.example.com \
  -e OCTANEST_RUNNER_REGISTRATION_TOKEN=... \
  -v octanest-runner-data:/data \
  octanest-runner
```

Add `-v /var/run/docker.sock:/var/run/docker.sock` for `docker://` labels —
that socket is root-equivalent on the host; isolate the runner from the API
process (see `docs/ARCHITECTURE.md`).

### Compose profile `actions`

The main stack ships a dev registration token so this works out of the box
locally (never reuse that default in production):

```bash
docker compose --profile actions up -d runner
# or everything incl. auth stubs:
make up-with-dev-auth
```

## Manual pipeline verification

`make seed-actions-demo` pushes a `ci-demo` repo + workflow and polls the
run to green — see `scripts/dev-auth/seed-actions-demo.sh`.
