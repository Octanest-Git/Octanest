#!/usr/bin/env bash
# Bring up Mailpit + OIDC mock + HTTP stubs, run API (SQLite) + Vite, execute Vitest stack e2e.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

COMPOSE="${COMPOSE:-docker compose}"
API_PORT="${OCTANEST_E2E_API_PORT:-18080}"
WEB_PORT="${OCTANEST_E2E_WEB_PORT:-13000}"
DB_PATH="${OCTANEST_E2E_DB_PATH:-$ROOT/var/e2e/octanest.db}"
API_PID=""
WEB_PID=""
STUBS_ONLY="${OCTANEST_E2E_STUBS_ONLY:-0}"

cleanup() {
  if [[ -n "${WEB_PID}" ]] && kill -0 "$WEB_PID" 2>/dev/null; then
    kill "$WEB_PID" 2>/dev/null || true
    wait "$WEB_PID" 2>/dev/null || true
  fi
  if [[ -n "${API_PID}" ]] && kill -0 "$API_PID" 2>/dev/null; then
    kill "$API_PID" 2>/dev/null || true
    wait "$API_PID" 2>/dev/null || true
  fi
  if [[ "${OCTANEST_E2E_KEEP_STUBS:-0}" != "1" && "$STUBS_ONLY" != "1" ]]; then
    $COMPOSE -f docker-compose.dev-auth.yml --profile dev-auth down --remove-orphans >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

# Prefer a credential-helper-free Docker config in CI/WSL when desktop helper is missing.
# shellcheck source=../docker-wsl-creds.sh
source "${ROOT}/scripts/docker-wsl-creds.sh"

need_cmd cargo
need_cmd bun
if ! command -v docker >/dev/null 2>&1; then
  echo "error: docker is required for Mailpit / OIDC mock / stubs" >&2
  exit 1
fi

mkdir -p "$(dirname "$DB_PATH")"
rm -f "$DB_PATH"

echo "==> starting dev-auth stubs (Mailpit, OIDC mock, HTTP stubs)"
$COMPOSE -f docker-compose.dev-auth.yml --profile dev-auth up --build -d

wait_http() {
  # Prefer IPv4; also try localhost↔127.0.0.1 so Vite/services bound to only one still pass.
  local url="$1" name="$2" tries="${3:-60}"
  local alt=""
  if [[ "$url" == *://127.0.0.1* ]]; then
    alt="${url//127.0.0.1/localhost}"
  elif [[ "$url" == *://localhost* ]]; then
    alt="${url//localhost/127.0.0.1}"
  fi
  for ((i = 1; i <= tries; i++)); do
    if curl -4 -fsS "$url" >/dev/null 2>&1; then
      echo "==> $name ready ($url)"
      return 0
    fi
    if [[ -n "$alt" ]] && curl -fsS "$alt" >/dev/null 2>&1; then
      echo "==> $name ready ($alt)"
      return 0
    fi
    sleep 1
  done
  echo "error: timed out waiting for $name at $url" >&2
  if [[ -n "$alt" ]]; then
    echo "error: also tried $alt" >&2
  fi
  exit 1
}

wait_http "http://127.0.0.1:8025/api/v1/info" "Mailpit"
wait_http "http://127.0.0.1:9092/health" "HTTP stubs"
wait_http "http://127.0.0.1:9090/default/.well-known/openid-configuration" "OIDC mock"

echo "==> building octanest-api"
cargo build -q -p octanest-api --bin octanest-api

echo "==> starting API on :$API_PORT (sqlite)"
export OCTANEST_ENV=development
export API_BIND="127.0.0.1:${API_PORT}"
export DATABASE_URL="sqlite:${DB_PATH}"
export OCTANEST_AUTO_MIGRATE=true
export OCTANEST_PUBLIC_ORIGIN="http://127.0.0.1:${WEB_PORT}"
export OCTANEST_CORS_ORIGINS="http://127.0.0.1:${WEB_PORT},http://localhost:${WEB_PORT},http://127.0.0.1:${API_PORT}"
export OCTANEST_MAIL_FROM="Octanest <noreply@localhost>"
export OCTANEST_SMTP_URL="smtp://127.0.0.1:1025"
export OCTANEST_RESEND_API_KEY="re_dev_local"
export OCTANEST_RESEND_BASE_URL="http://127.0.0.1:9092"
export WORKOS_API_KEY="sk_dev_local"
export WORKOS_CLIENT_ID="client_dev_local"
export OCTANEST_WORKOS_BASE_URL="http://127.0.0.1:9092"
export OCTANEST_OIDC_ALLOW_INSECURE=1
export OCTANEST_OIDC_ISSUER="http://127.0.0.1:9090/default"
export OCTANEST_OIDC_CLIENT_ID="octanest-dev"
export OCTANEST_OIDC_CLIENT_SECRET="octanest-dev"
export OCTANEST_ADMIN_EMAIL="admin@octanest.local"
export OCTANEST_ADMIN_PASSWORD="password1"
export RUST_LOG="${RUST_LOG:-info,octanest=debug}"

cargo run -q -p octanest-api --bin octanest-api >"$ROOT/var/e2e/api.log" 2>&1 &
API_PID=$!
wait_http "http://127.0.0.1:${API_PORT}/health" "octanest-api" 90

echo "==> starting Vite web on :$WEB_PORT (proxies /api → API)"
(
  cd apps/web
  # Point Vite proxy + SSR server fns at e2e API port
  export OCTANEST_E2E_API_ORIGIN="http://127.0.0.1:${API_PORT}"
  export OCTANEST_API_ORIGIN="http://127.0.0.1:${API_PORT}"
  export OCTANEST_PUBLIC_ORIGIN="http://127.0.0.1:${WEB_PORT}"
  # Bind IPv4 explicitly — default localhost can be ::1-only on CI, while we poll 127.0.0.1.
  bunx vite --host 127.0.0.1 --port "$WEB_PORT" --strictPort >"$ROOT/var/e2e/web.log" 2>&1
) &
WEB_PID=$!
wait_http "http://127.0.0.1:${WEB_PORT}/" "web" 90

export E2E_STACK=1
export OCTANEST_API_ORIGIN="http://127.0.0.1:${API_PORT}"
export OCTANEST_E2E_API_ORIGIN="http://127.0.0.1:${API_PORT}"
export OCTANEST_E2E_WEB_ORIGIN="http://127.0.0.1:${WEB_PORT}"
export OCTANEST_PUBLIC_ORIGIN="http://127.0.0.1:${WEB_PORT}"
export OCTANEST_E2E_MAILPIT_ORIGIN="http://127.0.0.1:8025"
export OCTANEST_E2E_STUBS_ORIGIN="http://127.0.0.1:9092"
export OCTANEST_E2E_OIDC_ISSUER="http://127.0.0.1:9090/default"
export OCTANEST_E2E_ADMIN_EMAIL="admin@octanest.local"
export OCTANEST_E2E_ADMIN_PASSWORD="password1"
export OCTANEST_E2E_DB_PATH="$DB_PATH"

echo "==> running Vitest stack e2e"
bun run --filter @octanest/web test:e2e:stack

echo "==> stack e2e passed"
