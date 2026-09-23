#!/usr/bin/env bash
# Run bun.webview browser PoC files one Bun process per file (Chrome singleton isolation).
# Requires E2E_STACK=1 and a live API/Vite stack (see make test-bun-poc-browser).
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
  echo "error: no browser PoC tests under $DIR/browser" >&2
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
echo "==> bun.webview PoC browser suite passed"
