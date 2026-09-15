#!/usr/bin/env bash
# Print the host.docker.internal target for the dev-auth compose stack.
#
# Docker Desktop resolves the `host-gateway` magic automatically. The
# `docker`→podman alias (podman machine) does not: there is no host gateway
# IP to substitute, so compose fails with `host containers internal IP address
# is empty`. On podman, the compose bridge gateway of `octanest_dev_auth` is
# reachable from inside containers and forwards to published host ports.
#
# Usage:
#   HOST_GATEWAY_IP="$(./scripts/dev-auth/host-gateway-ip.sh)"
#   export HOST_GATEWAY_IP
#   docker compose ... up
#
# Precedence: OCTANEST_HOST_GATEWAY_IP override → podman detection → "host-gateway".
set -euo pipefail

if [[ -n "${OCTANEST_HOST_GATEWAY_IP:-}" ]]; then
  printf '%s\n' "$OCTANEST_HOST_GATEWAY_IP"
  exit 0
fi

if command -v podman >/dev/null 2>&1 && podman network exists octanest_dev_auth 2>/dev/null; then
  gw="$(podman network inspect octanest_dev_auth --format '{{range .Subnets}}{{.Gateway}}{{end}}' 2>/dev/null || true)"
  if [[ -n "$gw" ]]; then
    printf '%s\n' "$gw"
    exit 0
  fi
fi

printf '%s\n' "host-gateway"
