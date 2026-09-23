#!/usr/bin/env bash
# Run all apps/web + packages/api-client unit-style tests under bun:test (dual-run with Vitest).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Run api-client tests first (simpler, no web dependencies)
cd "$ROOT/packages/api-client"
echo "==> bun test (api-client unit):"
if ! bun test --conditions=octanest-bun-test ./src; then
  echo "error: api-client tests failed" >&2
  exit 1
fi

# Run web tests
cd "$ROOT/apps/web"
mapfile -t files < <(
  find src -type f \( \
    -name '*.unit.test.ts' -o \
    -name '*.gate.test.ts' -o \
    -name 'markdown.test.ts' -o \
    -name 'markdown.issues.test.ts' -o \
    -name 'highlight.test.ts' \
  \) | sort
)
echo "==> bun test (web unit): ${#files[@]} files"
# Avoid apps/web/bunfig.toml happy-dom preload for pure unit (use Octane .tsrx stubs).
exec bun test --conditions=octanest-bun-test --preload ./bun-test/preload-unit.ts "${files[@]}"
