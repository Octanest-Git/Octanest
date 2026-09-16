#!/usr/bin/env bash
# Compose bring-up smoke (PLAT-01 / PLAT-08). Requires `docker` on PATH with a reachable engine.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker not found on PATH; cannot run compose smoke" >&2
  exit 1
fi
if ! docker info >/dev/null 2>&1; then
  echo "docker engine not reachable; cannot run compose smoke" >&2
  exit 1
fi
echo "==> using docker: $(command -v docker)"

BASE_URL="${OCTANEST_SMOKE_URL:-http://localhost}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
COMPOSE_FILES="${COMPOSE_FILES:--f $COMPOSE_FILE}"

# COMPOSE_PROFILES doesn't reliably get picked up in this WSL setup,
# so we translate it into explicit `--profile` flags.
PROFILES_ARGS=()
if [[ -n "${COMPOSE_PROFILES:-}" ]]; then
  for p in $(echo "${COMPOSE_PROFILES}" | tr ',' ' '); do
    [[ -n "$p" ]] || continue
    PROFILES_ARGS+=(--profile "$p")
  done
fi

EXPECT_DIALECT="${EXPECT_DIALECT:-postgres}"

# docker.exe (Windows client) does not reliably inherit WSL-exported env vars for
# Compose interpolation. Pass SQLite host dir via --env-file when set.
ENV_FILE_ARGS=()
if [[ -n "${OCTANEST_SQLITE_HOST_DIR:-}" ]]; then
  SMOKE_ENV_FILE="$(mktemp)"
  printf 'OCTANEST_SQLITE_HOST_DIR=%s\n' "$OCTANEST_SQLITE_HOST_DIR" > "$SMOKE_ENV_FILE"
  ENV_FILE_ARGS=(--env-file "$SMOKE_ENV_FILE")
fi

cleanup() {
  # shellcheck disable=SC2086
  docker compose "${ENV_FILE_ARGS[@]}" "${PROFILES_ARGS[@]}" $COMPOSE_FILES down --remove-orphans >/dev/null 2>&1 || true
}
cleanup_all() {
  if [[ -n "${SMOKE_ENV_FILE:-}" ]]; then rm -f "$SMOKE_ENV_FILE"; fi
  cleanup
}
trap cleanup_all EXIT

echo "==> docker compose config"
# shellcheck disable=SC2086
docker compose "${ENV_FILE_ARGS[@]}" "${PROFILES_ARGS[@]}" $COMPOSE_FILES config >/dev/null

echo "==> docker compose up --build -d --wait"
# shellcheck disable=SC2086
docker compose "${ENV_FILE_ARGS[@]}" "${PROFILES_ARGS[@]}" $COMPOSE_FILES up --build -d --wait

# Traefik docker-provider discovery can lag container healthchecks (--wait).
# Retry until routers are live so we don't flake with an immediate 404.
wait_http() {
  local url="$1"
  local label="$2"
  local deadline=$((SECONDS + 60))
  local code=""
  while (( SECONDS < deadline )); do
    code="$(curl -sS -o /dev/null -w '%{http_code}' "$url" || true)"
    if [[ "$code" =~ ^(200|301|302|303|307|308)$ ]]; then
      echo "$label $code"
      return 0
    fi
    sleep 2
  done
  echo "$label failed (last HTTP $code) after waiting for Traefik route: $url" >&2
  return 1
}

echo "==> curl web / (wait for Traefik)"
wait_http "$BASE_URL/" "web"

echo "==> curl /health via Traefik"
wait_http "$BASE_URL/health" "health"

echo "==> RPC system.health"
curl -fsS \
  -H 'Content-Type: application/json' \
  -H 'Octanest-RPC-Version: 1' \
  -d '{"procedure":"system.health","input":{}}' \
  "$BASE_URL/api/rpc" -o /tmp/octanest-smoke-rpc.json
grep -Eq '"ok"[[:space:]]*:[[:space:]]*true|"status"[[:space:]]*:[[:space:]]*"ok"' /tmp/octanest-smoke-rpc.json

echo "==> RPC system.db_probe (expect dialect=$EXPECT_DIALECT)"
curl -fsS \
  -H 'Content-Type: application/json' \
  -H 'Octanest-RPC-Version: 1' \
  -d '{"procedure":"system.db_probe","input":{}}' \
  "$BASE_URL/api/rpc" -o /tmp/octanest-smoke-probe.json
grep -Eq "\"dialect\"[[:space:]]*:[[:space:]]*\"$EXPECT_DIALECT\"" /tmp/octanest-smoke-probe.json
count_1="$(grep -o '"probe_count"[^,}]*' /tmp/octanest-smoke-probe.json | grep -o '[0-9]\+')"

curl -fsS \
  -H 'Content-Type: application/json' \
  -H 'Octanest-RPC-Version: 1' \
  -d '{"procedure":"system.db_probe","input":{}}' \
  "$BASE_URL/api/rpc" -o /tmp/octanest-smoke-probe.json
grep -Eq "\"dialect\"[[:space:]]*:[[:space:]]*\"$EXPECT_DIALECT\"" /tmp/octanest-smoke-probe.json
count_2="$(grep -o '"probe_count"[^,}]*' /tmp/octanest-smoke-probe.json | grep -o '[0-9]\+')"

if (( count_2 <= count_1 )); then
  echo "probe_count did not increase across calls ($count_1 -> $count_2)" >&2
  exit 1
fi
echo "==> dialect=$EXPECT_DIALECT probe_count $count_1 -> $count_2"

echo "==> smoke OK ($EXPECT_DIALECT)"
