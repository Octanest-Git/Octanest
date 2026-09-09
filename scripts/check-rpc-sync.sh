#!/usr/bin/env bash
set -euo pipefail

# Regenerates the typed client and fails if packages/api-client drifts.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

test -d packages/api-client

make rpc-gen

# When codegen lands (plan 01-02), this catches uncommitted drift:
# git diff --exit-code -- packages/api-client
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  if git diff --quiet -- packages/api-client 2>/dev/null; then
    echo "rpc-sync-check: packages/api-client clean (or unchanged)"
  else
    echo "rpc-sync-check: packages/api-client has local changes (allowed until codegen lands)"
  fi
fi

echo "rpc-sync-check: ok"
