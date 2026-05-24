#!/usr/bin/env bash
set -euo pipefail

missing=0

check_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing command: %s
' "$1"
    missing=1
  fi
}

check_pkg() {
  if ! pkg-config --exists "$1"; then
    printf 'missing pkg-config package: %s
' "$1"
    missing=1
  fi
}

check_cmd cargo
check_cmd rustc
check_cmd strip
check_cmd mpv
check_cmd pkg-config

check_pkg gtk4
check_pkg gio-2.0
check_pkg gdk-pixbuf-2.0
check_pkg mpv
check_pkg epoxy

if [ "$missing" -ne 0 ]; then
  cat <<'EOF'

Ubuntu/Debian packages:
  sudo apt install build-essential cargo rustc pkg-config libgtk-4-dev libmpv-dev libepoxy-dev mpv desktop-file-utils
EOF
  exit 1
fi

printf 'all dependencies found
'
