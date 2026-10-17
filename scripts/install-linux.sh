#!/usr/bin/env bash
# Installs an extracted Linux release tarball into ~/.local (no root needed).
#
# Usage:
#   tar -xzf waydroid-profile-manager_<version>_linux-x86_64.tar.gz
#   cd (extracted directory)
#   ./install.sh
#
# Override the prefix with PREFIX=/opt/waydroid-profile-manager ./install.sh
set -euo pipefail

SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
APP_DIR="$PREFIX/share/waydroid-profile-manager"

for required in bundle share/applications/com.waydroidpm.waydroid_profile_manager.desktop \
                share/icons/hicolor; do
  if [ ! -e "$SRC/$required" ]; then
    echo "error: $required not found — run this script from the extracted release" >&2
    exit 1
  fi
done

echo "==> Installing to $PREFIX"
mkdir -p "$BIN_DIR" "$APP_DIR" "$PREFIX/share/applications" "$PREFIX/share/icons"

# The runner locates data/flutter_assets relative to its own executable, so
# the whole bundle must stay together.
cp -a "$SRC/bundle/." "$APP_DIR/"
chmod +x "$APP_DIR/waydroid_profile_manager"
ln -sf "$APP_DIR/waydroid_profile_manager" "$BIN_DIR/waydroid_profile_manager"

cp "$SRC/share/applications/com.waydroidpm.waydroid_profile_manager.desktop" \
   "$PREFIX/share/applications/"
cp -a "$SRC/share/icons/hicolor/." "$PREFIX/share/icons/hicolor/"

# AppStream metadata (appdata) for GNOME Software / Discover / AppInstaller.
if [ -d "$SRC/share/metainfo" ]; then
  mkdir -p "$PREFIX/share/metainfo"
  cp -a "$SRC/share/metainfo/." "$PREFIX/share/metainfo/"
fi

# Best effort: refresh caches when the tools exist, ignore when they don't.
command -v update-desktop-database >/dev/null 2>&1 \
  && update-desktop-database "$PREFIX/share/applications" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null 2>&1 \
  && gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "note: add $BIN_DIR to your PATH to launch 'waydroid_profile_manager' directly" ;;
esac

echo "==> Installed: launch 'waydroid_profile_manager' or find"
echo "    'Waydroid Profile Manager' in your application menu."