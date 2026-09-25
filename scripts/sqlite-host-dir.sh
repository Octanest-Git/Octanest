#!/usr/bin/env bash
# Resolve a host directory Docker can bind-mount for SQLite (D-19).
#
# WSL + docker.exe cannot reliably bind Linux/WSL paths (distro mount socket
# missing → UNC \\wsl.localhost\... fails at container create). In that case
# return a Windows-native path. Native Linux Docker keeps using $ROOT/var.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$ROOT/var"

client_os="$(docker version --format '{{.Client.Os}}' 2>/dev/null || true)"

win_temp_dir() {
  local user="${OXIDEAN_WIN_USER:-}"
  if [[ -z "$user" ]] && command -v cmd.exe >/dev/null 2>&1; then
    user="$(cmd.exe /c 'echo %USERNAME%' 2>/dev/null | tr -d '\r')"
  fi
  user="${user:-Jesse}"
  local linux_path="/mnt/c/Users/${user}/AppData/Local/Temp/oxidean-sqlite-var"
  mkdir -p "$linux_path"
  # Docker Desktop on Windows wants a Windows path form.
  if command -v wslpath >/dev/null 2>&1; then
    wslpath -w "$linux_path" | sed 's|\\|/|g'
  else
    echo "C:/Users/${user}/AppData/Local/Temp/oxidean-sqlite-var"
  fi
}

if [[ "${client_os}" == "windows" ]]; then
  win_temp_dir
  exit 0
fi

echo "$ROOT/var"
