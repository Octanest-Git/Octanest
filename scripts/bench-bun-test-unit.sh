#!/usr/bin/env bash
# Benchmark Vitest unit slice vs bun:test PoC unit suite (issue #37).
# Writes JSON under var/bun-test/ (gitignored). Prints a markdown table to stdout.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/var/bun-test"
mkdir -p "$OUT"
N="${BENCH_RUNS:-3}"

run_timed() {
  local label="$1"
  shift
  local start end ms
  start=$(date +%s%N)
  "$@" >/dev/null 2>&1
  end=$(date +%s%N)
  ms=$(( (end - start) / 1000000 ))
  echo "$ms"
}

median() {
  local arr=("$@")
  local sorted
  mapfile -t sorted < <(printf '%s\n' "${arr[@]}" | sort -n)
  local mid=$(( ${#sorted[@]} / 2 ))
  if (( ${#sorted[@]} % 2 == 1 )); then
    echo "${sorted[$mid]}"
  else
    echo $(( (sorted[mid-1] + sorted[mid]) / 2 ))
  fi
}

echo "==> bench: Vitest unit (rpc-error-message, safe-external-url, public-origin) ×$N"
vitest_ms=()
for i in $(seq 1 "$N"); do
  ms=$(
    run_timed vitest bash -c "cd \"$ROOT/apps/web\" && bunx vitest run --project unit \
      src/lib/rpc-error-message.unit.test.ts \
      src/lib/safe-external-url.unit.test.ts \
      src/lib/public-origin.unit.test.ts"
  )
  vitest_ms+=("$ms")
  echo "  vitest run $i: ${ms}ms"
done

echo "==> bench: bun:test PoC unit ×$N"
bun_ms=()
for i in $(seq 1 "$N"); do
  ms=$(run_timed buntest bash "$ROOT/scripts/run-bun-test-unit.sh")
  bun_ms+=("$ms")
  echo "  bun:test run $i: ${ms}ms"
done

v_med=$(median "${vitest_ms[@]}")
b_med=$(median "${bun_ms[@]}")
v_min=$(printf '%s\n' "${vitest_ms[@]}" | sort -n | head -1)
v_max=$(printf '%s\n' "${vitest_ms[@]}" | sort -n | tail -1)
b_min=$(printf '%s\n' "${bun_ms[@]}" | sort -n | head -1)
b_max=$(printf '%s\n' "${bun_ms[@]}" | sort -n | tail -1)

ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
json="$OUT/unit-bench-$ts.json"
cat >"$json" <<EOF
{
  "timestamp": "$ts",
  "runs": $N,
  "vitest_ms": [$(IFS=,; echo "${vitest_ms[*]}")],
  "bun_test_ms": [$(IFS=,; echo "${bun_ms[*]}")],
  "vitest_median_ms": $v_med,
  "bun_test_median_ms": $b_med,
  "vitest_min_ms": $v_min,
  "vitest_max_ms": $v_max,
  "bun_test_min_ms": $b_min,
  "bun_test_max_ms": $b_max
}
EOF

echo
echo "| Runner | median (ms) | min | max |"
echo "|--------|-------------|-----|-----|"
echo "| Vitest unit (3 files) | $v_med | $v_min | $v_max |"
echo "| bun:test PoC unit | $b_med | $b_min | $b_max |"
echo
echo "Wrote $json"
