#!/usr/bin/env bash
# Interim E2E layer score for D-QH-02 (until stack-browser % coverage exists).
# Score = present checklist items / total. Expand when Phase 11.1-03 forge matrix lands.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
score_only=0
[[ "${1:-}" == "--score-only" ]] && score_only=1

# Interim checklist (auth + protocol smokes + stack HTTP e2e presence).
# Forge-matrix + admin quota/auth suites (11.1-04 / 11.1-07 / 11.1-08).
ITEMS=(
  "apps/web/e2e/stack-browser/auth-ui.stack.browser.test.tsx"
  "apps/web/e2e/stack-browser/forge-admin.stack.browser.test.tsx"
  "apps/web/e2e/stack/smtp.stack.test.ts"
  "apps/web/e2e/stack/oidc.stack.test.ts"
  "scripts/smoke-git-https.sh"
  "scripts/smoke-git-ssh.sh"
  "scripts/smoke-packages.sh"
)

total="${#ITEMS[@]}"
present=0
missing=()
for rel in "${ITEMS[@]}"; do
  if [[ -f "$ROOT/$rel" ]]; then
    present=$((present + 1))
  else
    missing+=("$rel")
  fi
done

score="$(bun -e 'process.stdout.write((Number(process.argv[1])/Number(process.argv[2])).toFixed(6))' "$present" "$total")"

if [[ "$score_only" -eq 1 ]]; then
  printf '%s\n' "$score"
  exit 0
fi

echo "coverage-e2e-checklist: $present/$total present → score=$score"
if [[ ${#missing[@]} -gt 0 ]]; then
  echo "coverage-e2e-checklist: missing:"
  printf '  - %s\n' "${missing[@]}"
fi
exit 0
