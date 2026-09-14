#!/usr/bin/env bash
# Compose Git LFS smoke (GIT-12 / D-LFS-05 / D-LFS-06).
# Documents Traefik .git/info/lfs routing intent: batch + basic transfer
# under /{owner}/{repo}.git/info/lfs/… (HTTPS only; LFS-over-SSH deferred).
#
# Prerequisites (when Docker available):
#   - Docker Compose stack up (`make up`) with Traefik on :80
#   - `git` + `git-lfs` on PATH
#   - Repo with LFS enabled + PAT with write scope for push
#
# Env knobs:
#   OCTANEST_SMOKE_URL   default http://localhost
#   SMOKE_GIT_OWNER      default smokeowner
#   SMOKE_GIT_REPO        default smokerepo
#   SMOKE_PAT            optional; required for push portion
#
# CI / hosts without Docker: exits 0 with a skip message (mirror smoke-git-https).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BASE_URL="${OCTANEST_SMOKE_URL:-http://localhost}"
OWNER="${SMOKE_GIT_OWNER:-smokeowner}"
REPO="${SMOKE_GIT_REPO:-smokerepo}"
GIT_URL="${BASE_URL}/${OWNER}/${REPO}.git"
LFS_BATCH="${GIT_URL}/info/lfs/objects/batch"

if ! command -v git >/dev/null 2>&1; then
  echo "git not found on PATH; cannot run git LFS smoke" >&2
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "docker not found on PATH; skipping smoke-git-lfs (operator/CI without Compose)"
  exit 0
fi
if ! docker info >/dev/null 2>&1; then
  echo "docker engine not reachable; skipping smoke-git-lfs"
  exit 0
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

echo "==> Traefik LFS batch routing (must not be SPA text/html): ${LFS_BATCH}"
tmp_headers="$(mktemp)"
tmp_body="$(mktemp)"
trap 'rm -f "$tmp_headers" "$tmp_body"' EXIT

http_code="$(
  curl -sS -D "$tmp_headers" -o "$tmp_body" -w "%{http_code}" \
    -H "Host: localhost" \
    -H "Accept: application/vnd.git-lfs+json" \
    -H "Content-Type: application/vnd.git-lfs+json" \
    -X POST \
    -d '{"operation":"download","transfers":["basic"],"objects":[{"oid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","size":1}]}' \
    "$LFS_BATCH" || true
)"
ct="$(grep -i '^content-type:' "$tmp_headers" | head -1 | tr -d '\r' || true)"
body_head="$(head -c 200 "$tmp_body" | tr '\n' ' ')"

if echo "$ct" | grep -qi 'text/html'; then
  echo "FAIL: .git/info/lfs path returned text/html (SPA stole LFS). content-type=$ct code=$http_code" >&2
  echo "body: $body_head" >&2
  exit 1
fi

echo "LFS batch probe: http=$http_code content-type=$ct"
echo "OK: Traefik routes .git/info/lfs away from SPA (full push/pull needs LFS-enabled repo + PAT)"
# Full git-lfs push/pull is operator-exercised once Phase 14 handlers land;
# Wave 0 only asserts routing + docker-skip behavior.
exit 0
