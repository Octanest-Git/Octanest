#!/usr/bin/env bash
# Compose Git-over-SSH smoke (GIT-03 / D-SSH-02 / D-SSH-07).
# Wave 0 stub: discoverable Make target; greened by 09-05 (ls-remote/push over TCP 2222).
#
# Prerequisites (when greened):
#   - Docker Compose stack up with SSH listener on OCTANEST_SSH_PORT (default 2222)
#   - `git` + `ssh` on PATH
#   - Registered SSH key for SMOKE_GIT_OWNER; public repo SMOKE_GIT_OWNER/SMOKE_GIT_REPO
#   - scp-style remote: git@host:owner/repo.git (D-SSH-02) — not ssh:// primary
#
# Env knobs (future green):
#   OCTANEST_SSH_HOST   default localhost
#   OCTANEST_SSH_PORT   default 2222
#   SMOKE_GIT_OWNER     default smokeowner
#   SMOKE_GIT_REPO       default smokerepo
#
# CI / hosts without Docker: exits 0 with a skip message when docker is missing
# (same spirit as smoke-git-https.sh).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SSH_HOST="${OCTANEST_SSH_HOST:-localhost}"
SSH_PORT="${OCTANEST_SSH_PORT:-2222}"
OWNER="${SMOKE_GIT_OWNER:-smokeowner}"
REPO="${SMOKE_GIT_REPO:-smokerepo}"
# scp-style (D-SSH-02); Port via SSH config / -p when ≠ 22
GIT_SSH_URL="git@${SSH_HOST}:${OWNER}/${REPO}.git"

if ! command -v git >/dev/null 2>&1; then
  echo "git not found on PATH; cannot run git SSH smoke" >&2
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "docker not found on PATH; skipping smoke-git-ssh (operator/CI without Compose)"
  exit 0
fi
if ! docker info >/dev/null 2>&1; then
  echo "docker engine not reachable; skipping smoke-git-ssh"
  exit 0
fi

# Wave 0: Docker present but SSH Compose publish + ls-remote/push not wired until 09-05.
echo "Wave 0: smoke-git-ssh stub — TCP ${SSH_PORT} + ls-remote/push for ${GIT_SSH_URL} not greened yet (09-05)" >&2
exit 1
