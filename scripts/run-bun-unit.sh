#!/usr/bin/env bash
# Run all apps/web unit-style tests under bun:test (dual-run with Vitest).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
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
# Avoid apps/web/bunfig.toml happy-dom preload for pure unit (use Octane .tsrx stubs).
exec bun test --conditions=octanest-bun-test --preload ./bun-test/preload-unit.ts "${files[@]}"
