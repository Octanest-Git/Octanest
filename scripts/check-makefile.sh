#!/usr/bin/env bash
# Parse the root Makefile (GNU Make) and run checkmake.
# Fail closed: syntax errors, undefined-variable warnings, and checkmake violations.
#
# Usage: make makefile-lint
#        ./scripts/check-makefile.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CHECKMAKE_VERSION="${CHECKMAKE_VERSION:-0.3.2}"
MAKEFILE="${MAKEFILE:-Makefile}"
CONFIG="${CHECKMAKE_CONFIG:-checkmake.ini}"

die() {
  echo "error: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "'$1' is required"
}

need_cmd make
need_cmd curl
need_cmd uname
need_cmd mkdir
need_cmd mktemp

[[ -f "$MAKEFILE" ]] || die "Makefile not found: $MAKEFILE"
[[ -f "$CONFIG" ]] || die "checkmake config not found: $CONFIG"

# --- 1) GNU Make parse + undefined-variable warnings (dry-run help) ---
warn_log="$(mktemp)"
trap 'rm -f "$warn_log"' EXIT

# Expand the help recipes; discard recipe echo, keep make diagnostics on stderr.
if ! make -n --warn-undefined-variables -f "$MAKEFILE" help >/dev/null 2>"$warn_log"; then
  echo "==> GNU Make failed to parse or dry-run '$MAKEFILE'" >&2
  cat "$warn_log" >&2
  exit 1
fi

if grep -F 'warning: undefined variable' "$warn_log" >/dev/null 2>&1; then
  echo "==> undefined Make variables while expanding '$MAKEFILE' (help):" >&2
  grep -F 'warning: undefined variable' "$warn_log" >&2 || true
  exit 1
fi
echo "makefile-lint: GNU Make parse ok"

# --- 2) checkmake (pinned binary under tmp/) ---
os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$arch" in
  x86_64 | amd64) arch=amd64 ;;
  aarch64 | arm64) arch=arm64 ;;
  *) die "unsupported arch for checkmake binary: $arch" ;;
esac
case "$os" in
  linux | darwin) ;;
  *) die "unsupported OS for checkmake binary: $os" ;;
esac

bin_dir="$ROOT/tmp/tools"
bin="$bin_dir/checkmake-v${CHECKMAKE_VERSION}.${os}.${arch}"
url="https://github.com/checkmake/checkmake/releases/download/v${CHECKMAKE_VERSION}/checkmake-v${CHECKMAKE_VERSION}.${os}.${arch}"

if [[ ! -x "$bin" ]]; then
  mkdir -p "$bin_dir"
  echo "==> fetching checkmake v${CHECKMAKE_VERSION} ($os/$arch)"
  curl -fsSL -o "$bin" "$url"
  chmod +x "$bin"
fi

"$bin" --config "$CONFIG" "$MAKEFILE"
echo "makefile-lint: checkmake ok"
