#!/usr/bin/env bash
# Compose ORG-06 protected-push smoke (D-PKG-01 / D-PKG-03).
# Brings up a fresh Compose stack (wipes project volumes), asserts the protection
# helper binary is executable, creates a reviews-required rule with
# enforce_admins, and expects an HTTPS push to the protected ref to fail.
#
# Env:
#   OCTANEST_SMOKE_URL   default http://localhost
#   COMPOSE_FILE         default docker-compose.yml
#   SMOKE_REQUIRE_STACK  if 1 (or CI=true), fail closed when Docker missing
#
# Operator hosts without Docker: exits 0 with a skip message (unless fail-closed).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=scripts/smoke-lib.sh
source "${ROOT}/scripts/smoke-lib.sh"
SMOKE_NAME="compose-smoke-protection"

BASE_URL="${OCTANEST_SMOKE_URL:-http://localhost}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
OWNER="${SMOKE_PROTECT_OWNER:-protowner}"
REPO="${SMOKE_PROTECT_REPO:-protrepo}"
PASSWORD="${SMOKE_PROTECT_PASSWORD:-ProtectSmoke1!}"

smoke_require_docker

if ! command -v git >/dev/null 2>&1; then
  echo "git not found on PATH; cannot run protection smoke" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found on PATH; cannot parse RPC JSON" >&2
  exit 1
fi

cleanup() {
  docker compose -f "$COMPOSE_FILE" down --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> docker compose down -v (clean volumes for bootstrap)"
docker compose -f "$COMPOSE_FILE" down -v --remove-orphans >/dev/null 2>&1 || true

echo "==> docker compose config"
docker compose -f "$COMPOSE_FILE" config >/dev/null

if [[ "${OCTANEST_COMPOSE_SKIP_BUILD:-}" == "1" ]]; then
  echo "==> docker compose up -d --wait (OCTANEST_COMPOSE_SKIP_BUILD=1)"
  docker compose -f "$COMPOSE_FILE" up -d --wait
else
  echo "==> docker compose up --build -d --wait"
  docker compose -f "$COMPOSE_FILE" up --build -d --wait
fi

echo "==> wait for ${BASE_URL}/health"
ok=0
for _ in $(seq 1 90); do
  if curl -fsS -o /dev/null "${BASE_URL}/health" 2>/dev/null; then
    ok=1
    break
  fi
  sleep 2
done
if [[ "$ok" -ne 1 ]]; then
  echo "FAIL: health check failed at ${BASE_URL}/health" >&2
  docker compose -f "$COMPOSE_FILE" ps >&2 || true
  exit 1
fi

echo "==> assert protection helper executable in API image"
docker compose -f "$COMPOSE_FILE" exec -T api test -x /usr/local/bin/octanest-protection-hook
echo "==> helper OK"

COOKIE_JAR="$(mktemp)"
RPC_OUT="$(mktemp)"
WORK="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -f '$COOKIE_JAR' '$RPC_OUT'; rm -rf '$WORK'; cleanup" EXIT

rpc() {
  local body="$1"
  local http_code
  http_code="$(
    curl -sS -o "$RPC_OUT" -w "%{http_code}" \
      -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
      -H "content-type: application/json" \
      -H "Octanest-RPC-Version: 1" \
      -d "$body" \
      "${BASE_URL}/api/rpc" || true
  )"
  if [[ "$http_code" != "200" ]]; then
    echo "RPC HTTP $http_code for: $body" >&2
    head -c 800 "$RPC_OUT" >&2 || true
    echo >&2
    return 1
  fi
  if python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if d.get("ok") is True else 1)' "$RPC_OUT"; then
    return 0
  fi
  echo "RPC error for: $body" >&2
  head -c 800 "$RPC_OUT" >&2 || true
  echo >&2
  return 1
}

rpc_json_field() {
  python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d["data"][sys.argv[2]])' "$RPC_OUT" "$1"
}

echo "==> bootstrap verified admin (${OWNER})"
rpc "$(python3 -c 'import json; print(json.dumps({"procedure":"auth.bootstrap_setup","input":{"email":"'"$OWNER"'@example.com","username":"'"$OWNER"'","password":"'"$PASSWORD"'","allow_signup":False,"provider_mode":"local"}}))')"

echo "==> create public repo ${OWNER}/${REPO}"
rpc "$(python3 -c 'import json; print(json.dumps({"procedure":"repo.create","input":{"name":"'"$REPO"'","visibility":"public","description":"protection smoke","stack_id":"rust","license_id":"MIT","gitignore_id":"Rust"}}))')"

echo "==> create classic PAT"
rpc '{"procedure":"pat.createClassic","input":{"name":"protect-smoke","scopes":["repo"]}}'
PAT="$(rpc_json_field token)"
if [[ -z "$PAT" || "$PAT" == "None" ]]; then
  echo "FAIL: pat.createClassic did not return token" >&2
  exit 1
fi

echo "==> create branch protection (reviews + enforce_admins) on main"
rpc "$(python3 -c 'import json; print(json.dumps({"procedure":"repo.branchProtection.create","input":{"owner":"'"$OWNER"'","name":"'"$REPO"'","pattern":"main","require_reviews":True,"required_approving_review_count":1,"enforce_admins":True}}))')"

AUTH_HOST="${BASE_URL#http://}"
AUTH_HOST="${AUTH_HOST#https://}"
AUTH_URL="http://git:${PAT}@${AUTH_HOST}/${OWNER}/${REPO}.git"

echo "==> clone + attempt protected push to main (expect denial)"
git -C "$WORK" clone -q "$AUTH_URL" repo
git -C "$WORK/repo" config user.email "protect-smoke@localhost"
git -C "$WORK/repo" config user.name "protect-smoke"
echo "protect-smoke $(date -u +%Y%m%dT%H%M%SZ)" >>"$WORK/repo/PROTECT_SMOKE.txt"
git -C "$WORK/repo" add PROTECT_SMOKE.txt
git -C "$WORK/repo" commit -q -m "protect-smoke direct push"

set +e
push_out="$(GIT_TERMINAL_PROMPT=0 git -C "$WORK/repo" push origin HEAD:refs/heads/main 2>&1)"
push_rc=$?
set -e

if [[ "$push_rc" -eq 0 ]]; then
  echo "FAIL: HTTPS push to protected main succeeded (expected denial)" >&2
  echo "$push_out" >&2
  exit 1
fi
echo "==> push denied as expected (rc=$push_rc)"
echo "$push_out" | head -20 || true

echo "==> compose-smoke-protection OK (helper present + HTTPS protected push denied)"
