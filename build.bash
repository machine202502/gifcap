#!/usr/bin/env bash
# Build the GIF-only release. Set VCPKG_ROOT and LIBCLANG_PATH first.
set -euo pipefail

if [[ -z "${VCPKG_ROOT:-}" ]]; then
  echo "error: set VCPKG_ROOT to the directory with vcpkg.exe" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TRIPLET="x64-windows-static-md-release"
INSTALL_ROOT="$ROOT/vcpkg_installed"

win_path() {
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$1"
  else
    printf '%s' "$1"
  fi
}

export VCPKG_OVERLAY_PORTS="$(win_path "$ROOT/vcpkg-overlays")"
export VCPKG_OVERLAY_TRIPLETS="$(win_path "$ROOT/triplets")"
export FFMPEG_DIR="$(win_path "$INSTALL_ROOT/$TRIPLET")"

cd "$ROOT"
"$VCPKG_ROOT/vcpkg.exe" install \
  --triplet "$TRIPLET" \
  --overlay-ports "$VCPKG_OVERLAY_PORTS" \
  --overlay-triplets "$VCPKG_OVERLAY_TRIPLETS" \
  --x-manifest-root="$(win_path "$ROOT")" \
  --x-install-root="$(win_path "$INSTALL_ROOT")"

cargo build --release -p gifcap
echo "done: target/x86_64-pc-windows-msvc/release/gifcap.exe"
