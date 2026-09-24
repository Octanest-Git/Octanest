#!/usr/bin/env bash
# e2e-actions-pipeline.sh — full Actions pipeline e2e with NO Docker:
#   API (sqlite) + octanest-runner (host mode) + seed demo workflow → green run.
#
# Usage: ./scripts/e2e-actions-pipeline.sh
# Requires: cargo, git, curl, jq.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

API_PORT="${OCTANEST_E2E_API_PORT:-18081}"
API_ORIGIN="http://127.0.0.1:${API_PORT}"
REG_TOKEN="${OCTANEST_RUNNER_REGISTRATION_TOKEN:-e2e-dev-runner-token}"
WORK="$(mktemp -d)"
API_PID=""
RUNNER_PID=""

cleanup() {
  for pid in "$RUNNER_PID" "$API_PID"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$WORK"
}
trap cleanup EXIT

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "error: required command not found: $1" >&2; exit 1; }
}
need_cmd cargo; need_cmd git; need_cmd curl; need_cmd jq

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

echo "==> building octanest-api + octanest-runner"
cargo build -q -p octanest-api --bin octanest-api -p octanest-runner --bin octanest-runner

echo "==> starting API on :$API_PORT (sqlite, actions enabled)"
export OCTANEST_ENV=development
export API_BIND="127.0.0.1:${API_PORT}"
export DATABASE_URL="sqlite:${WORK}/octanest.db"
export OCTANEST_AUTO_MIGRATE=true
export OCTANEST_PUBLIC_ORIGIN="$API_ORIGIN"
export OCTANEST_CORS_ORIGINS="$API_ORIGIN"
export OCTANEST_REPOS_DIR="${WORK}/repos"
export OCTANEST_ACTIONS_ENABLED=true
export OCTANEST_ACTIONS_LOG_DIR="${WORK}/actions-logs"
export OCTANEST_ACTIONS_SECRETS_KEY="e2e-actions-secrets-key"
export OCTANEST_RUNNER_REGISTRATION_TOKEN="$REG_TOKEN"
export OCTANEST_ADMIN_EMAIL="admin@octanest.local"
export OCTANEST_ADMIN_PASSWORD="password1"
export RUST_LOG="${RUST_LOG:-info,octanest=debug}"

"$ROOT/target/debug/octanest-api" >"$WORK/api.log" 2>&1 &
API_PID=$!
wait_http "$API_ORIGIN/health" "octanest-api" 90

echo "==> starting octanest-runner (host mode)"
export OCTANEST_PUBLIC_ORIGIN="$API_ORIGIN"
export OCTANEST_RUNNER_NAME="e2e-runner"
export OCTANEST_RUNNER_LABELS="ubuntu-latest"
export OCTANEST_RUNNER_STATE="${WORK}/runner.json"
export OCTANEST_RUNNER_WORK_DIR="${WORK}/runner-work"
export OCTANEST_RUNNER_POLL_MS=500

"$ROOT/target/debug/octanest-runner" >"$WORK/runner.log" 2>&1 &
RUNNER_PID=$!
sleep 1
kill -0 "$RUNNER_PID" 2>/dev/null || { echo "runner exited early:"; cat "$WORK/runner.log"; exit 1; }

echo "==> seeding ci-demo + pushing workflow"
OCTANEST_ORIGIN="$API_ORIGIN" \
OCTANEST_SEED_USER="admin@octanest.local" \
OCTANEST_SEED_PASSWORD="password1" \
OCTANEST_SEED_TIMEOUT_S="${OCTANEST_SEED_TIMEOUT_S:-120}" \
OCTANEST_SEED_REQUIRE_MARKER=1 \
  ./scripts/dev-auth/seed-actions-demo.sh

echo "==> PASS — pipeline run green end to end"
