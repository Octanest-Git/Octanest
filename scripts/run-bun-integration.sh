#!/usr/bin/env bash
# Dual-run happy-dom lib integration under bun:test.
#
# Residual (Vitest-only until Bun can load Octane `.tsrx`):
# - route/component suites under src/routes and src/components
# - src/lib/session-cache.integration.test.ts (needs QueryClientProvider.tsrx)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/apps/web"
mapfile -t files < <(
  find src/lib -type f -name '*.integration.test.ts' \
    ! -name 'session-cache.integration.test.ts' \
    | sort
)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "error: no dual-run lib integration tests found" >&2
  exit 1
fi

# Run tests in parallel, each in its own process for isolation
pids=()
for f in "${files[@]}"; do
  echo "==> bun test (parallel integration lib): $f"
  bun test --conditions=octanest-bun-test --preload ./bun-test/preload-web.ts "$f" &
  pids+=($!)
done

# Wait for all tests and collect exit codes
failed=0
for pid in "${pids[@]}"; do
  if ! wait "$pid"; then
    failed=1
  fi
done

exit "$failed"
