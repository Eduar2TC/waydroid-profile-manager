#!/usr/bin/env bash
#
# Builds every Linux release artifact, each one carrying the desktop icon:
#
#   dist/waydroid-profile-manager_<version>_linux-x86_64.tar.gz
#   dist/waydroid-profile-manager_<version>_amd64.deb
#   dist/WaydroidProfileManager-<version>-x86_64.AppImage
#   dist/SHA256SUMS
#
# Usage:
#   scripts/package-linux.sh                 # build bundle, then all formats
#   scripts/package-linux.sh --skip-build    # reuse an existing release bundle
#   scripts/package-linux.sh --only tar      # tar | deb | appimage
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PKG_NAME="waydroid-profile-manager"
BIN_NAME="waydroid_profile_manager"
ICON_NAME="waydroidpm"
DESKTOP="linux/packaging/com.waydroidpm.waydroid_profile_manager.desktop"
DESKTOP_BASE="$(basename "$DESKTOP")"
ICONS_SRC="linux/packaging/icons"
METAINFO_SRC="linux/packaging/metainfo/com.waydroidpm.waydroid_profile_manager.appdata.xml"
BUNDLE="build/linux/x64/release/bundle"
DIST="$ROOT/dist"
WORK="$ROOT/build/packaging"

SKIP_BUILD=0
ONLY="all"
while [ $# -gt 0 ]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1 ;;
    --only) ONLY="${2:?--only needs a format}"; shift ;;
    --only=*) ONLY="${1#--only=}" ;;
    -h|--help) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "error: unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

case "$ONLY" in all|tar|deb|appimage) ;; *)
  echo "error: --only must be tar, deb or appimage (got: $ONLY)" >&2; exit 2 ;;
esac

# pubspec version "1.0.0+1" -> "1.0.0"
VERSION="$(sed -n 's/^version:[[:space:]]*\([0-9][0-9.]*\).*/\1/p' pubspec.yaml | head -1)"
if [ -z "$VERSION" ]; then
  echo "error: could not read version from pubspec.yaml" >&2
  exit 1
fi

if [ "$SKIP_BUILD" -eq 0 ]; then
  echo "==> flutter build linux --release"
  flutter build linux --release
fi

if [ ! -x "$BUNDLE/$BIN_NAME" ]; then
  echo "error: release bundle not found at $BUNDLE — run 'flutter build linux --release'" >&2
  exit 1
fi

if command -v desktop-file-validate >/dev/null 2>&1; then
  desktop-file-validate "$DESKTOP"
  echo "==> $DESKTOP is valid"
fi

# Fails the build if the AppStream metadata regresses (soft-gates on the tool
# being installed; CI installs `appstream` so the check really runs there).
if command -v appstreamcli >/dev/null 2>&1; then
  appstreamcli validate --no-net "$METAINFO_SRC"
  echo "==> $METAINFO_SRC is valid"
else
  echo "==> appstreamcli not installed, skipping AppStream validation"
fi

rm -rf "$WORK"
STAGE="$WORK/stage"
mkdir -p "$STAGE/share/applications" "$STAGE/share/icons" \
         "$STAGE/share/metainfo" "$DIST"
cp -a "$BUNDLE" "$STAGE/bundle"
cp "$DESKTOP" "$STAGE/share/applications/"
cp -a "$ICONS_SRC/hicolor" "$STAGE/share/icons/hicolor"
cp "$METAINFO_SRC" "$STAGE/share/metainfo/"
chmod +x "$STAGE/bundle/$BIN_NAME"

# ---------------------------------------------------------------- tar.gz ----
build_tar() {
  echo "==> tar.gz"
  cp "$ROOT/scripts/install-linux.sh" "$STAGE/install.sh"
  chmod +x "$STAGE/install.sh"
  tar -czf "$DIST/${PKG_NAME}_${VERSION}_linux-x86_64.tar.gz" \
      -C "$STAGE" bundle share install.sh

}

# ------------------------------------------------------------------ .deb ----
build_deb() {
  if ! command -v dpkg-deb >/dev/null 2>&1; then
    echo "error: dpkg-deb not found (Debian/Ubuntu/dpkg-based system required)." >&2
    echo "       Build a single format with --only tar" >&2
    exit 1
  fi
  echo "==> .deb"
  local root="$WORK/debroot"
  mkdir -p "$root/DEBIAN" "$root/usr/lib/$PKG_NAME" "$root/usr/bin" \
           "$root/usr/share/applications" "$root/usr/share/icons" \
           "$root/usr/share/metainfo"
  # The runner resolves data/flutter_assets next to its own executable, so the
  # bundle stays intact under /usr/lib and /usr/bin points into it.
  cp -a "$BUNDLE" "$root/usr/lib/$PKG_NAME/bundle"
  ln -sf "../lib/$PKG_NAME/bundle/$BIN_NAME" "$root/usr/bin/$BIN_NAME"
  cp "$DESKTOP" "$root/usr/share/applications/"
  cp -a "$ICONS_SRC/hicolor" "$root/usr/share/icons/hicolor"
  cp "$METAINFO_SRC" "$root/usr/share/metainfo/"

  local installed_size
  installed_size="$(du -sk "$root" | cut -f1)"
  cat > "$root/DEBIAN/control" <<CTRL
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Eduar2TC <Eduar2TC@users.noreply.github.com>
Installed-Size: ${installed_size}
Depends: libgtk-3-0, libayatana-appindicator3-1
Description: Waydroid profile manager with display profiles
 Applies resolution, density and rotation profiles to a running Waydroid
 session, restoring the previous configuration on reset.
CTRL
  dpkg-deb --root-owner-group --build "$root" \
      "$DIST/${PKG_NAME}_${VERSION}_amd64.deb"
}

# -------------------------------------------------------------- AppImage ----
build_appimage() {
  local tool="$WORK/appimagetool-x86_64.AppImage"
  mkdir -p "$WORK"
  echo "==> AppImage"
  if [ ! -x "$tool" ]; then
    echo "    downloading appimagetool"
    if ! curl -fsSL -o "$tool" \
        https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage; then
      echo "error: could not download appimagetool (network down?)." >&2
      echo "       Build a single format with --only tar|deb" >&2
      rm -f "$tool"
      exit 1
    fi
    chmod +x "$tool"
  fi

  local appdir="$WORK/AppDir"
  # usr/share/metainfo is where appimagetool looks for AppStream metadata;
  # without it the tool warns and software centres cannot list the app.
  # appstreamcli (run by appimagetool) also needs the .desktop file inside
  # the AppDir's standard layout, not only at the AppDir root.
  mkdir -p "$appdir/usr/share/metainfo" "$appdir/usr/share/applications"
  cp -a "$BUNDLE" "$appdir/bundle"
  cp "$DESKTOP" "$appdir/$DESKTOP_BASE"
  cp "$DESKTOP" "$appdir/usr/share/applications/"
  cp "$ICONS_SRC/$ICON_NAME.png" "$appdir/$ICON_NAME.png"
  cp "$METAINFO_SRC" "$appdir/usr/share/metainfo/"
  cat > "$appdir/AppRun" <<'APPRUN'
#!/bin/sh
# Resolve the bundle next to AppRun so the app finds its own data dir.
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/bundle/waydroid_profile_manager" "$@"
APPRUN
  chmod +x "$appdir/AppRun"

  # --appimage-extract-and-run: works without FUSE (CI runners, containers).
  ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$tool" \
      "$appdir" "$DIST/WaydroidProfileManager-${VERSION}-x86_64.AppImage"
}

case "$ONLY" in
  tar)      build_tar ;;
  deb)      build_deb ;;
  appimage) build_appimage ;;
  all)      build_tar; build_deb; build_appimage ;;
esac

# ------------------------------------------------------------- checksums ----)
(
  cd "$DIST"
  : > SHA256SUMS
  for f in *.tar.gz *.deb *.AppImage; do
    [ -e "$f" ] && sha256sum "$f" >> SHA256SUMS
  done
)

echo
echo "==> Artifacts in dist/:"
ls -lh "$DIST" | tail -n +2