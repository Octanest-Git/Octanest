#!/usr/bin/env bash
# Compose bring-up smoke (PLAT-01). Requires Docker Engine available.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if command -v docker >/dev/null 2>&1; then
  DOCKER=(docker)
elif [[ -x "/mnt/c/Program Files/Docker/Docker/resources/bin/docker.exe" ]]; then
  DOCKER=("/mnt/c/Program Files/Docker/Docker/resources/bin/docker.exe")
else
  echo "docker not found; cannot run compose smoke" >&2
  exit 1
fi

BASE_URL="${OCTANEST_SMOKE_URL:-http://localhost}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"

cleanup() {
  "${DOCKER[@]}" compose -f "$COMPOSE_FILE" down --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> docker compose config"
"${DOCKER[@]}" compose -f "$COMPOSE_FILE" config >/dev/null

echo "==> docker compose up --build -d --wait"
"${DOCKER[@]}" compose -f "$COMPOSE_FILE" up --build -d --wait

echo "==> curl web /"
curl -fsS -o /dev/null -w "web %{http_code}\n" "$BASE_URL/"

echo "==> curl /healthz via Traefik"
curl -fsS -o /dev/null -w "healthz %{http_code}\n" "$BASE_URL/healthz"

echo "==> RPC system.health"
curl -fsS \
  -H "Content-Type: application/json" \
  -H "Octanest-RPC-Version: 1" \
  -d '{"procedure":"system.health","input":{}}' \
  "$BASE_URL/api/rpc" | tee /tmp/octanest-smoke-rpc.json
grep -q '"ok":true\|"status":"ok"' /tmp/octanest-smoke-rpc.json

echo "==> smoke OK"
