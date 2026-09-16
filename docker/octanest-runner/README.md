# Octanest official Actions runner

Image wrapping [Gitea `act_runner`](https://gitea.com/gitea/act_runner) so operators supply compute without Octanest-managed minutes (ACT-04 / ACT-05 / ACT-07).

**Base image:** `gitea/act_runner:0.2.11` (pinned in `Dockerfile` — bump deliberately).

## Register

Never register with a loopback URL if job containers need to checkout from the forge (Gitea “Connection 2” lesson). Use the published hostname:

```bash
export OCTANEST_PUBLIC_ORIGIN=https://git.example.com
export OCTANEST_RUNNER_REGISTRATION_TOKEN=...   # Admin / instance bootstrap; never commit
export OCTANEST_RUNNER_NAME=runner-1
export OCTANEST_RUNNER_LABELS='ubuntu-latest:docker://node:20-bookworm,self-hosted'
```

### Standalone

```bash
docker build -t octanest-runner -f docker/octanest-runner/Dockerfile docker/octanest-runner

docker run --rm -it \
  -e OCTANEST_PUBLIC_ORIGIN \
  -e OCTANEST_RUNNER_REGISTRATION_TOKEN \
  -e OCTANEST_RUNNER_NAME \
  -e OCTANEST_RUNNER_LABELS \
  -v octanest-runner-data:/data \
  -v /var/run/docker.sock:/var/run/docker.sock \
  octanest-runner
```

Docker socket is required for `label:docker://…` jobs; isolate the runner host from the API process (T-19-18).

### Compose profile `actions`

```bash
export OCTANEST_RUNNER_REGISTRATION_TOKEN=...
docker compose --profile actions up -d runner
```

See root `docker-compose.yml` service `runner` and [docs/DEPLOYMENT.md](../../docs/DEPLOYMENT.md).

## Labels

Default labels include `ubuntu-latest:docker://node:20-bookworm` so workflows with `runs-on: ubuntu-latest` match (D-ACT-09). Customize via `OCTANEST_RUNNER_LABELS`.

## Protocol

Octanest exposes JSON runner endpoints under `/api/actions` (`Register`, `Declare`, `FetchTask`, `UpdateTask`, `UpdateLog`). Registration tokens and runner bearer tokens only — session cookies are ignored (D-ACT-18).

## Secrets

- Pass `OCTANEST_RUNNER_REGISTRATION_TOKEN` via env or secret file — **never** commit real tokens.
- Rotate tokens from Admin when compromised (D-ACT-08).
