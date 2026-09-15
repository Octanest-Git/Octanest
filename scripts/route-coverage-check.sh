#!/usr/bin/env bash
# Route coverage gate (G-11.1-15 / Phase 11.1-08).
# Every user-facing apps/web/src/routes/**/*.tsrx page must appear in
# apps/web/src/test/route-coverage.manifest.ts with happy-dom, stack-browser,
# or a documented skip. Outlet-only layouts / __root marked layoutOnly are
# excluded from the required set.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$ROOT/scripts/route-coverage-check.ts"

[[ -f "$SCRIPT" ]] || {
  echo "route-coverage-check: FAIL: missing $SCRIPT" >&2
  exit 1
}
command -v bun >/dev/null 2>&1 || {
  echo "route-coverage-check: FAIL: bun is required" >&2
  exit 1
}

exec bun "$SCRIPT"
