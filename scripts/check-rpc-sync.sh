#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
test -d packages/api-client
make rpc-gen
git diff --exit-code -- packages/api-client
echo "rpc-sync-check: ok"
