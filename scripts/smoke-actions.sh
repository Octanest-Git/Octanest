#!/usr/bin/env bash
# Compose Actions / Runners smoke (ACT-04 / ACT-05 / D-ACT-11 / D-ACT-14).
# Placeholder until 19-08/19-11 greens: runner profile, actions health, protocol path.
#
# Prerequisites (when fully greened):
#   - Docker Compose stack up (`make up`) with Traefik on :80
#   - Optional Compose profile `actions` / official runner sidecar
#
# Env knobs:
#   OCTANEST_SMOKE_URL   default http://localhost
#
# Operator hosts without Docker/stack: exits 0 with a skip message.
# CI=true or SMOKE_REQUIRE_STACK=1 fails closed (T-11.1-40 / D-QH-04).
# Never prints registration tokens or other secrets.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=scripts/smoke-lib.sh
source "${ROOT}/scripts/smoke-lib.sh"
SMOKE_NAME="smoke-actions"

BASE_URL="${OCTANEST_SMOKE_URL:-http://localhost}"

smoke_require_docker
# Fast skip when Compose API isn't up (avoid long health wait).
if ! docker compose -f docker-compose.yml ps --status running 2>/dev/null | grep -qE 'api|octanest-api'; then
  smoke_require_or_skip "Compose API not running; skipping smoke-actions (run make up to exercise Actions)"
fi

echo "==> wait for ${BASE_URL}/health"
ok=0
for _ in $(seq 1 60); do
  if curl -fsS -o /dev/null "${BASE_URL}/health" 2>/dev/null; then
    ok=1
    break
  fi
  sleep 2
done
if [[ "$ok" -ne 1 ]]; then
  echo "health check failed at ${BASE_URL}/health — is the stack up? (make up)" >&2
  exit 1
fi

# Wave 0 placeholder — 19-08/19-11 will assert runner profile / protocol paths.
echo "==> Actions smoke placeholder (Wave 0) — stack healthy at ${BASE_URL}"
echo "SKIP: full runner registration + job dispatch checks land in 19-08/19-11"
exit 0
