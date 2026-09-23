#!/usr/bin/env bash
# Run bun.webview browser tests with process isolation (sequential for CI stability).
# Each test file runs in its own Bun process (Chrome singleton isolation).
# Requires E2E_STACK=1 and a live API/Vite stack (see make test-bun-browser).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/apps/web/bun-test"
cd "$DIR"

if [[ "${E2E_STACK:-}" != "1" ]]; then
  echo "error: E2E_STACK=1 required" >&2
  exit 1
fi

shopt -s nullglob
files=("$DIR"/browser/*.stack.browser.test.ts)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "error: no browser tests under $DIR/browser" >&2
  exit 1
fi

failed=0
for f in "${files[@]}"; do
  echo "==> bun test (isolated process): $(basename "$f")"
  if ! bun test --isolate "$f"; then
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi
echo "==> bun.webview browser suite passed"
