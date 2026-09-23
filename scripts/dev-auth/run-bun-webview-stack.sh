#!/usr/bin/env bash
# Bring up the same stack as run-stack-e2e.sh, then run bun.webview browser tests.
# Primary browser test path (issue #37).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export BUN_TEST=1
export OCTANEST_E2E_KEEP_STUBS="${OCTANEST_E2E_KEEP_STUBS:-0}"

# Reuse stack bring-up by wrapping: run harness with a custom test command.
# Inline a minimal fork of run-stack-e2e that swaps the Vitest invocation.
COMPOSE="${COMPOSE:-docker compose}"
API_PORT="${OCTANEST_E2E_API_PORT:-18080}"
WEB_PORT="${OCTANEST_E2E_WEB_PORT:-13000}"
DB_PATH="${OCTANEST_E2E_DB_PATH:-$ROOT/var/e2e/octanest.db}"
API_PID=""
WEB_PID=""

cleanup() {
  if [[ -n "${WEB_PID}" ]] && kill -0 "$WEB_PID" 2>/dev/null; then
    kill "$WEB_PID" 2>/dev/null || true
    wait "$WEB_PID" 2>/dev/null || true
  fi
  if [[ -n "${API_PID}" ]] && kill -0 "$API_PID" 2>/dev/null; then
    kill "$API_PID" 2>/dev/null || true
    wait "$API_PID" 2>/dev/null || true
  fi
  if [[ "${OCTANEST_E2E_KEEP_STUBS:-0}" != "1" ]]; then
    $COMPOSE -f docker-compose.dev-auth.yml --profile dev-auth down --remove-orphans >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

# shellcheck source=../docker-wsl-creds.sh
source "${ROOT}/scripts/docker-wsl-creds.sh"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

need_cmd cargo
need_cmd bun
need_cmd docker

mkdir -p "$(dirname "$DB_PATH")" "$ROOT/var/bun-test"
rm -f "$DB_PATH"

echo "==> starting dev-auth stubs"
$COMPOSE -f "$ROOT/docker-compose.dev-auth.yml" --profile dev-auth up --build -d

wait_http() {
  local url="$1" name="$2" tries="${3:-60}"
  for ((i = 1; i <= tries; i++)); do
    if curl -4 -fsS "$url" >/dev/null 2>&1; then
      echo "==> $name ready ($url)"
      return 0
    fi
    sleep 1
  done
  echo "error: timed out waiting for $name at $url" >&2
  exit 1
}

wait_http "http://127.0.0.1:8025/api/v1/info" "Mailpit"
wait_http "http://127.0.0.1:9092/health" "HTTP stubs"
wait_http "http://127.0.0.1:9090/default/.well-known/openid-configuration" "OIDC mock"

echo "==> building octanest-api"
cargo build -q -p octanest-api --bin octanest-api

echo "==> starting API on :$API_PORT"
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

echo "==> starting Vite web on :$WEB_PORT"
(
  cd "$ROOT/apps/web"
  export OCTANEST_E2E_API_ORIGIN="http://127.0.0.1:${API_PORT}"
  export OCTANEST_API_ORIGIN="http://127.0.0.1:${API_PORT}"
  export OCTANEST_PUBLIC_ORIGIN="http://127.0.0.1:${WEB_PORT}"
  bunx vite --host 127.0.0.1 --port "$WEB_PORT" --strictPort >"$ROOT/var/e2e/web.log" 2>&1
) &
WEB_PID=$!
wait_http "http://127.0.0.1:${WEB_PORT}/" "web" 90

echo "==> warming auth routes"
for path in /signup /login /status; do
  for _ in 1 2 3 4 5; do
    code=$(curl -4 -sS -o /dev/null -w "%{http_code}" "http://127.0.0.1:${WEB_PORT}${path}" || echo 000)
    if [[ "$code" =~ ^(200|302|303|307|308)$ ]]; then
      break
    fi
    sleep 2
  done
done

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

echo "==> running bun:test e2e/stack HTTP dual-run"
bash "$ROOT/scripts/run-bun-e2e-stack.sh"

echo "==> running bun.webview browser suite"
bash "$ROOT/scripts/run-bun-webview.sh"

echo "==> bun:test stack (HTTP + WebView) passed"
