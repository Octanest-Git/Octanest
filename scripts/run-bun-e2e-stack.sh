#!/usr/bin/env bash
# Dual-run e2e/stack HTTP tests under bun:test (requires E2E_STACK=1 + live stack).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/apps/web"
if [[ "${E2E_STACK:-}" != "1" ]]; then
  echo "error: E2E_STACK=1 required (use make test-bun-unit-browser / stack harness)" >&2
  exit 1
fi
mapfile -t files < <(find e2e/stack -type f -name '*.stack.test.ts' | sort)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "error: no e2e/stack HTTP tests found" >&2
  exit 1
fi
failed=0
for f in "${files[@]}"; do
  echo "==> bun test (e2e HTTP): $f"
  if ! bun test --conditions=octanest-bun-test --preload ./e2e/stack/setup.ts "$f"; then
    failed=1
  fi
done
exit "$failed"
