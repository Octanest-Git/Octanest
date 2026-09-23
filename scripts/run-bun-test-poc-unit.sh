#!/usr/bin/env bash
# Run bun:test PoC unit suite (no WebView / no Docker).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/apps/web/bun-test"
exec bun test ./unit
