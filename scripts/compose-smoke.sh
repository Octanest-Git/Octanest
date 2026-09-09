#!/usr/bin/env bash
# Compose bring-up smoke (PLAT-01). Requires Docker Engine available.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pick_docker() {
  local candidates=(
    "/mnt/c/Program Files/Docker/Docker/resources/bin/docker.exe"
    docker
    /mnt/wsl/docker-desktop/cli-tools/usr/bin/docker
  )

  local c
  for c in "${candidates[@]}"; do
    if [[ "$c" == docker ]]; then
      command -v docker >/dev/null 2>&1 || continue
    else
      [[ -x "$c" ]] || continue
    fi
    if "$c" info >/dev/null 2>&1; then
      DOCKER=("$c")
      return 0
    fi
  done
  return 1
}

if ! pick_docker; then
  echo "docker engine not reachable; cannot run compose smoke" >&2
  exit 1
fi
echo "==> using docker: ${DOCKER[*]}"
# docker.exe from WSL cannot resolve docker-credential-desktop on Windows PATH.
# Use a temp config without credsStore for pulls/builds.
CFG_WSL="/mnt/c/Users/Jesse/AppData/Local/Temp/octanest-docker-config"
mkdir -p "$CFG_WSL"
printf '%s\n' '{' '  "auths": {},' '  "currentContext": "desktop-linux"' '}' > "$CFG_WSL/config.json"
export DOCKER_CONFIG='C:\Users\Jesse\AppData\Local\Temp\octanest-docker-config'
export PATH="/mnt/c/Program Files/Docker/Docker/resources/bin:${PATH}"

# Windows docker.exe needs Desktop bin on PATH for docker-credential-desktop
export PATH="/mnt/c/Program Files/Docker/Docker/resources/bin:${PATH}"


BASE_URL="${OCTANEST_SMOKE_URL:-http://localhost}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
COMPOSE_FILES="${COMPOSE_FILES:--f $COMPOSE_FILE}"
EXPECT_DIALECT="${EXPECT_DIALECT:-postgres}"

cleanup() {
  "${DOCKER[@]}" compose $COMPOSE_FILES down --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> docker compose config"
"${DOCKER[@]}" compose $COMPOSE_FILES config >/dev/null

echo "==> docker compose up --build -d --wait"
"${DOCKER[@]}" compose $COMPOSE_FILES up --build -d --wait

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
