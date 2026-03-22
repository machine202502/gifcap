#!/usr/bin/env bash
# Сборка gifcap slim: FFmpeg только GIF + скриншоты PNG; без MP4/WebP в vcpkg и в UI.
# Перед запуском задай только:
#   export VCPKG_ROOT=/c/path/to/vcpkg
#   export LIBCLANG_PATH="/c/Program Files/.../Llvm/x64/bin"

set -euo pipefail

if [[ -z "${VCPKG_ROOT:-}" ]]; then
  echo "error: задай VCPKG_ROOT — каталог с vcpkg.exe" >&2
  exit 1
fi
if [[ -z "${LIBCLANG_PATH:-}" ]]; then
  echo "error: задай LIBCLANG_PATH — каталог с libclang.dll" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TRIPLET="x64-windows-static-md-release"
INSTALL_ROOT="$ROOT/vcpkg_installed_slim"

win_path() {
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$1"
  else
    printf '%s' "$1"
  fi
}

export VCPKG_OVERLAY_PORTS="$(win_path "$ROOT/vcpkg-overlays/slim")"
export FFMPEG_DIR="$(win_path "$INSTALL_ROOT/$TRIPLET")"
WIN_ROOT="$(win_path "$ROOT")"
WIN_INSTALL="$(win_path "$INSTALL_ROOT")"

cd "$ROOT"
echo "== vcpkg install (overlay: slim, install-root: vcpkg_installed_slim) =="
echo "   VCPKG_OVERLAY_PORTS=$VCPKG_OVERLAY_PORTS"
"$VCPKG_ROOT/vcpkg.exe" install \
  --triplet "$TRIPLET" \
  --x-no-default-features \
  --x-manifest-root="$WIN_ROOT" \
  --x-install-root="$WIN_INSTALL"

echo "== cargo clean =="
cargo clean

echo "== cargo build --release -p gifcap --features slim =="

export CARGO_PROFILE_RELEASE_OPT_LEVEL=z 
export CARGO_PROFILE_RELEASE_LTO=fat 
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 
export CARGO_PROFILE_RELEASE_PANIC=abort 
export CARGO_PROFILE_RELEASE_STRIP=symbols
export CARGO_PROFILE_RELEASE_DEBUG=0

cargo build --release -p gifcap --features slim

echo "готово: target/x86_64-pc-windows-msvc/release/gifcap.exe"
