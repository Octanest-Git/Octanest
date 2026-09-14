#!/usr/bin/env bash
# Prefer a credential-helper-free Docker config in WSL when Docker Desktop's
# helper (`docker-credential-desktop.exe`) is referenced but not on PATH.
# Safe to `source` from Makefiles / scripts. No-op when DOCKER_CONFIG is set
# or the helper is available.
#
# Usage:
#   source "$(git rev-parse --show-toplevel)/scripts/docker-wsl-creds.sh"
#   docker compose ...

if [[ -n "${DOCKER_CONFIG:-}" ]]; then
  return 0 2>/dev/null || exit 0
fi

if command -v docker-credential-desktop.exe >/dev/null 2>&1 \
  || command -v docker-credential-desktop >/dev/null 2>&1; then
  return 0 2>/dev/null || exit 0
fi

_cfg="${HOME}/.docker/config.json"
if [[ ! -f "$_cfg" ]]; then
  return 0 2>/dev/null || exit 0
fi

# Docker Desktop on WSL often writes `"credsStore": "desktop.exe"`.
if ! grep -qE 'desktop\.exe|docker-credential-desktop' "$_cfg" 2>/dev/null; then
  return 0 2>/dev/null || exit 0
fi

_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export DOCKER_CONFIG="${_root}/var/docker-config-wsl"
mkdir -p "$DOCKER_CONFIG"
printf '%s\n' '{}' >"$DOCKER_CONFIG/config.json"
unset _cfg _root
