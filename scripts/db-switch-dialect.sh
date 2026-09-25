#!/usr/bin/env bash
# Guarded dialect-switch helper (D-03, D-16): migrates DATABASE_URL only if it
# is empty. Never echoes the URL itself — only the detected dialect (T-02-12).
set -euo pipefail

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "set DATABASE_URL to the NEW dialect target before running" >&2
  exit 1
fi

case "$DATABASE_URL" in
  postgres://* | postgresql://*) dialect="postgres" ;;
  mysql://*) dialect="mysql" ;;
  sqlite:*) dialect="sqlite" ;;
  *) dialect="unknown" ;;
esac

echo "==> switching to dialect: $dialect"

cargo run -q -p oxidean-db --bin migrate -- --assert-empty "$@"

echo "==> migrated $dialect target"
echo "==> next step: persist DATABASE_URL in .env for future runs"
echo "==> note: Phase 2 supports switching only an empty target (D-03) — no data is copied between dialects"
