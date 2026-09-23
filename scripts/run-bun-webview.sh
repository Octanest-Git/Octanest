#!/usr/bin/env bash
# Run bun.webview browser tests with bounded parallelism and process isolation.
# Each test file runs in its own Bun process (Chrome singleton isolation).
# Workers are limited to prevent resource exhaustion in CI.
# Requires E2E_STACK=1 and a live API/Vite stack (see make test-bun-browser).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/apps/web/bun-test"
cd "$DIR"

if [[ "${E2E_STACK:-}" != "1" ]]; then
  echo "error: E2E_STACK=1 required" >&2
  exit 1
fi

# Bounded parallelism: default 2 workers in CI, 4 locally
# Override with BUN_WEBVIEW_WORKERS=N
if [[ "${CI:-}" = "true" ]]; then
  MAX_WORKERS="${BUN_WEBVIEW_WORKERS:-2}"
else
  MAX_WORKERS="${BUN_WEBVIEW_WORKERS:-4}"
fi

shopt -s nullglob
files=("$DIR"/browser/*.stack.browser.test.ts)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "error: no browser tests under $DIR/browser" >&2
  exit 1
fi

echo "==> Running ${#files[@]} WebView test files with $MAX_WORKERS isolated workers"

# Auth-sensitive tests that modify global auth settings must run sequentially first
auth_tests=(
  "auth-me-dedupe.stack.browser.test.ts"
  "signup.stack.browser.test.ts"
)

failed=0

# Run auth-sensitive tests sequentially first
for auth_test in "${auth_tests[@]}"; do
  auth_file="$DIR/browser/$auth_test"
  if [[ -f "$auth_file" ]]; then
    echo "==> bun test (auth-sensitive, sequential): $auth_test"
    if ! bun test --isolate "$auth_file"; then
      failed=1
    fi
    # Remove from files array
    files=("${files[@]/$auth_file}")
  fi
done

# Run remaining tests with bounded parallelism
for f in "${files[@]}"; do
  [[ -z "$f" ]] && continue  # Skip empty entries from array filtering
  rel="${f#"$DIR"/}"
  echo "==> bun test (isolated worker): $rel"
  
  # Spawn isolated worker process
  bun test --isolate "$f" &
  
  # Limit concurrent workers - wait for slot if at capacity
  while (( $(jobs -r | wc -l) >= MAX_WORKERS )); do
    wait -n || failed=1
  done
done

# Wait for all remaining workers
while (( $(jobs -r | wc -l) > 0 )); do
  wait -n || failed=1
done

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi
echo "==> bun.webview browser suite passed"
