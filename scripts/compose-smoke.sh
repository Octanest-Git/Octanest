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
EXPECT_DIALECT="${EXPECT_DIALECT:-postgres}"

cleanup() {
  # shellcheck disable=SC2086
  docker compose $COMPOSE_FILES down --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> docker compose config"
# shellcheck disable=SC2086
docker compose $COMPOSE_FILES config >/dev/null

echo "==> docker compose up --build -d --wait"
# shellcheck disable=SC2086
docker compose $COMPOSE_FILES up --build -d --wait

echo "==> curl web /"
curl -fsS -o /dev/null -w "web %{http_code}\n" "$BASE_URL/"

echo "==> curl /health via Traefik"
curl -fsS -o /dev/null -w "health %{http_code}\n" "$BASE_URL/health"

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
