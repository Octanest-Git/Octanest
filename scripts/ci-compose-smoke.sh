#!/usr/bin/env bash
# CI fail-closed entrypoint for Compose dialect bring-up smokes (D-CI-01…04).
# Reuses make smoke / smoke-sqlite / smoke-mysql → scripts/compose-smoke.sh.
# Does not rewrite dialect assertions; fail-closed when Docker is unavailable.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CI="${CI:-true}"
export SMOKE_REQUIRE_STACK="${SMOKE_REQUIRE_STACK:-1}"

DIALECT="${1:-postgres}"

case "$DIALECT" in
  postgres)
    echo "==> ci-compose-smoke: make smoke (EXPECT_DIALECT=postgres)"
    make smoke
    ;;
  sqlite)
    echo "==> ci-compose-smoke: make smoke-sqlite (EXPECT_DIALECT=sqlite)"
    # Native Linux Docker on GHA: sqlite-host-dir.sh resolves to $ROOT/var.
    make smoke-sqlite
    ;;
  mysql)
    echo "==> ci-compose-smoke: make smoke-mysql (EXPECT_DIALECT=mysql)"
    make smoke-mysql
    ;;
  *)
    echo "FAIL: unknown dialect '$DIALECT' (expected postgres|sqlite|mysql)" >&2
    exit 1
    ;;
esac

echo "==> ci-compose-smoke OK ($DIALECT)"
