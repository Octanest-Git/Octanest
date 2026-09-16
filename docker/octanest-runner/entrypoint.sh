#!/bin/sh
# Register (once) then daemon against OCTANEST_PUBLIC_ORIGIN /api/actions.
set -eu

ORIGIN="${OCTANEST_PUBLIC_ORIGIN:-}"
TOKEN="${OCTANEST_RUNNER_REGISTRATION_TOKEN:-}"
NAME="${OCTANEST_RUNNER_NAME:-octanest-runner}"
LABELS="${OCTANEST_RUNNER_LABELS:-ubuntu-latest:docker://node:20-bookworm,self-hosted}"
CONFIG="${OCTANEST_RUNNER_CONFIG:-/data/.runner}"

if [ -z "$ORIGIN" ]; then
  echo "OCTANEST_PUBLIC_ORIGIN is required (browser/network reachable forge URL — not localhost from job containers)" >&2
  exit 1
fi

ORIGIN="${ORIGIN%/}"

if [ ! -f "$CONFIG" ]; then
  if [ -z "$TOKEN" ]; then
    echo "OCTANEST_RUNNER_REGISTRATION_TOKEN required for first-time register" >&2
    exit 1
  fi
  echo "Registering runner ${NAME} against ${ORIGIN} ..."
  act_runner register \
    --instance "${ORIGIN}" \
    --token "${TOKEN}" \
    --name "${NAME}" \
    --labels "${LABELS}" \
    --no-interactive
fi

exec act_runner daemon
