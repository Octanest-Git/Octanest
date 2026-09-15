#!/usr/bin/env bash
# Compose Git LFS smoke (GIT-12 / D-LFS-05 / D-LFS-06).
# Traefik routes /{owner}/{repo}.git/info/lfs → API; optional git-lfs push/pull
# over HTTPS with PAT Basic (LFS-over-SSH is NOT tested — deferred).
#
# Prerequisites (when Docker available):
#   - Docker Compose stack up (`make up`) with Traefik on :80
#   - `git` on PATH; `git-lfs` for full push/pull
#   - Public (or PAT-accessible) repo at SMOKE_GIT_OWNER/SMOKE_GIT_REPO
#   - Repo with LFS enabled (Settings or SMOKE_SESSION + RPC setEnabled)
#   - SMOKE_PAT for push/pull (username=git; never printed)
#
# Env knobs:
#   OCTANEST_SMOKE_URL   default http://localhost
#   SMOKE_GIT_OWNER      default smokeowner
#   SMOKE_GIT_REPO        default smokerepo
#   SMOKE_PAT            required for push/pull portion
#   SMOKE_SESSION        optional Cookie header value to enable LFS via RPC
#   SMOKE_SKIP_LFS_CLIENT if 1, only assert Traefik LFS routing
#
# Operator hosts without Docker: exits 0 with a skip message.
# CI=true or SMOKE_REQUIRE_STACK=1 fails closed (T-11.1-40 / D-QH-04).
# git-lfs client / SMOKE_PAT remain optional after routing OK.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=scripts/smoke-lib.sh
source "${ROOT}/scripts/smoke-lib.sh"
SMOKE_NAME="smoke-git-lfs"

BASE_URL="${OCTANEST_SMOKE_URL:-http://localhost}"
OWNER="${SMOKE_GIT_OWNER:-smokeowner}"
REPO="${SMOKE_GIT_REPO:-smokerepo}"
GIT_URL="${BASE_URL}/${OWNER}/${REPO}.git"
LFS_BATCH="${GIT_URL}/info/lfs/objects/batch"
# Host for embedding credentials — never echo SMOKE_PAT.
HOST_PART="${BASE_URL#http://}"
HOST_PART="${HOST_PART#https://}"

if ! command -v git >/dev/null 2>&1; then
  echo "git not found on PATH; cannot run git LFS smoke" >&2
  exit 1
fi

smoke_require_docker

echo "==> wait for ${BASE_URL}/health"
ok=0
for _ in $(seq 1 3); do
  if curl -fsS --connect-timeout 1 --max-time 2 -o /dev/null "${BASE_URL}/health" 2>/dev/null; then
    ok=1
    break
  fi
  sleep 1
done
if [[ "$ok" -ne 1 ]]; then
  smoke_require_or_skip "stack health not reachable at ${BASE_URL}/health; skipping smoke-git-lfs (run make up first)"
fi

echo "==> Traefik LFS batch routing (must not be SPA text/html): ${LFS_BATCH}"
tmp_headers="$(mktemp)"
tmp_body="$(mktemp)"
trap 'rm -f "$tmp_headers" "$tmp_body"' EXIT

http_code="$(
  curl -sS --connect-timeout 2 --max-time 10 -D "$tmp_headers" -o "$tmp_body" -w "%{http_code}" \
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
if echo "$body_head" | grep -qiE '<(!doctype|html|script)'; then
  echo "FAIL: LFS response body looks like HTML (code=$http_code ct=$ct)" >&2
  echo "body: $body_head" >&2
  exit 1
fi
echo "==> routing OK (http=$http_code content-type=${ct:-none})"

if [[ "${SMOKE_SKIP_LFS_CLIENT:-0}" == "1" ]]; then
  echo "==> SMOKE_SKIP_LFS_CLIENT=1 — skipping git-lfs client"
  echo "==> smoke-git-lfs OK (routing only)"
  exit 0
fi

if ! command -v git-lfs >/dev/null 2>&1 && ! git lfs version >/dev/null 2>&1; then
  echo "git-lfs not found; skipping client push/pull (routing already OK)"
  exit 0
fi

if [[ -z "${SMOKE_PAT:-}" ]]; then
  echo "SMOKE_PAT unset; skipping git-lfs push/pull (routing already OK)"
  echo "Hint: enable LFS on ${OWNER}/${REPO}, set SMOKE_PAT=octanest_pat_… (redacted), re-run."
  exit 0
fi

# Optional: enable LFS via session RPC (never log cookie value).
if [[ -n "${SMOKE_SESSION:-}" ]]; then
  echo "==> enabling LFS via session RPC (cookie redacted)"
  enable_body='{"procedure":"repo.lfs.setEnabled","input":{"owner":"'"$OWNER"'","name":"'"$REPO"'","enabled":true}}'
  enable_code="$(
    curl -sS -o /tmp/octanest-lfs-enable.json -w "%{http_code}" \
      -H "Content-Type: application/json" \
      -H "Octanest-RPC-Version: 1" \
      -H "Cookie: ${SMOKE_SESSION}" \
      -d "$enable_body" \
      "${BASE_URL}/api/rpc" || true
  )"
  if [[ "$enable_code" != "200" ]] || grep -q '"error"' /tmp/octanest-lfs-enable.json 2>/dev/null; then
    echo "WARN: repo.lfs.setEnabled failed (http=$enable_code); continuing if already enabled" >&2
  else
    echo "==> LFS enabled"
  fi
fi

work="$(mktemp -d)"
clone="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$work' '$clone'; rm -f '$tmp_headers' '$tmp_body' /tmp/octanest-lfs-enable.json" EXIT

auth_url="http://git:${SMOKE_PAT}@${HOST_PART}/${OWNER}/${REPO}.git"

echo "==> clone + git lfs track / push (PAT redacted in logs)"
GIT_TERMINAL_PROMPT=0 git clone -q "$auth_url" "$work" \
  || {
    echo "git clone failed — create public repo ${OWNER}/${REPO} first" >&2
    exit 1
  }
git -C "$work" config user.email "smoke-lfs@localhost"
git -C "$work" config user.name "smoke-lfs"
git -C "$work" lfs install --local
git -C "$work" lfs track "*.bin"
# Ensure .gitattributes is committed by the client (D-LFS-17 — server never auto-commits).
echo "lfs-smoke $(date -u +%Y%m%dT%H%M%SZ)" >"$work/smoke-payload.bin"
head -c 256 /dev/urandom >>"$work/smoke-payload.bin"
git -C "$work" add .gitattributes smoke-payload.bin
git -C "$work" commit -q -m "smoke-git-lfs"
GIT_TERMINAL_PROMPT=0 git -C "$work" push -q origin "HEAD:refs/heads/smoke-lfs" \
  || {
    echo "git lfs push failed — is LFS enabled on the repo? (Settings → Git LFS)" >&2
    exit 1
  }
echo "==> push OK (refs/heads/smoke-lfs)"

echo "==> fresh clone + lfs pull"
GIT_TERMINAL_PROMPT=0 git clone -q -b smoke-lfs "$auth_url" "$clone" \
  || {
    echo "fresh clone failed" >&2
    exit 1
  }
GIT_TERMINAL_PROMPT=0 git -C "$clone" lfs pull \
  || {
    echo "git lfs pull failed" >&2
    exit 1
  }
if [[ ! -f "$clone/smoke-payload.bin" ]]; then
  echo "FAIL: smoke-payload.bin missing after lfs pull" >&2
  exit 1
fi
# Pointer file would be tiny; smudged object should be larger than a pointer.
sz="$(wc -c <"$clone/smoke-payload.bin" | tr -d ' ')"
if [[ "$sz" -lt 100 ]]; then
  echo "FAIL: expected smudged LFS bytes, got size=$sz (pointer leak?)" >&2
  exit 1
fi
echo "==> lfs pull OK (bytes=$sz)"

echo "==> smoke-git-lfs OK (HTTPS LFS only; LFS-over-SSH not claimed)"
