#!/usr/bin/env bash
# Promote instance_auth_settings.email_provider log → smtp for Compose+Mailpit.
# DB defaults to `log`, which overrides OXIDEAN_SMTP_URL at API boot — without
# this step verify mail stays in the API log sink and never reaches Mailpit.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# shellcheck source=../docker-wsl-creds.sh
source "${ROOT}/scripts/docker-wsl-creds.sh"

COMPOSE_FILES=(
  -f docker-compose.yml
  -f docker-compose.dev-auth.yml
  -f docker-compose.dev-auth-attach.yml
)
COMPOSE=(docker compose "${COMPOSE_FILES[@]}" --profile dev-auth)

echo "==> Waiting for postgres…"
for _ in $(seq 1 30); do
  if "${COMPOSE[@]}" exec -T postgres pg_isready -U "${POSTGRES_USER:-oxidean}" -d "${POSTGRES_DB:-oxidean}" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

BEFORE=$("${COMPOSE[@]}" exec -T postgres \
  psql -U "${POSTGRES_USER:-oxidean}" -d "${POSTGRES_DB:-oxidean}" -Atc \
  "SELECT email_provider FROM instance_auth_settings WHERE id = 1;" 2>/dev/null || echo "")

if [[ "$BEFORE" != "log" ]]; then
  echo "==> Auth email_provider is '${BEFORE:-unknown}' (not log) — leaving unchanged"
  exit 0
fi

"${COMPOSE[@]}" exec -T postgres \
  psql -U "${POSTGRES_USER:-oxidean}" -d "${POSTGRES_DB:-oxidean}" -c \
  "UPDATE instance_auth_settings SET email_provider = 'smtp', updated_at = now() WHERE id = 1 AND email_provider = 'log';" \
  >/dev/null

echo "==> Promoted email_provider log → smtp (Mailpit)"
"${COMPOSE[@]}" restart api >/dev/null
echo "==> Restarted api to pick up SMTP sender"
