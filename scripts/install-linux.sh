#!/usr/bin/env bash
# LocalSendStudio installer for Linux.
#
#   curl -fsSL https://raw.githubusercontent.com/unupunct/LocalSendStudio/main/scripts/install-linux.sh | bash
#
# Options (pass after `bash -s --` when piping):
#   --version vX.Y.Z    install a specific release instead of the latest
#   --appimage          force the AppImage even on Debian/Fedora systems
#   --open-firewall     open the ports LocalSendStudio needs (ufw or firewalld)
#   --uninstall         remove LocalSendStudio
#   -y                  do not ask questions
set -euo pipefail

REPO="unupunct/LocalSendStudio"
APP="LocalSendStudio"
VERSION=""
FORCE_APPIMAGE=0
OPEN_FW=0
UNINSTALL=0
YES=0

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="${2:?--version needs a value}"; shift 2 ;;
    --appimage) FORCE_APPIMAGE=1; shift ;;
    --open-firewall) OPEN_FW=1; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    -y|--yes) YES=1; shift ;;
    -h|--help) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
done

say()  { printf '\033[1;36m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!!\033[0m  %s\n' "$*" >&2; }
die()  { printf '\033[1;31mxx\033[0m  %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }
SUDO=""; [ "$(id -u)" -ne 0 ] && have sudo && SUDO="sudo"

[ "$(uname -s)" = "Linux" ] || die "This installer is for Linux. See docs/INSTALL.md for other systems."

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) DEB_ARCH="amd64"; RPM_ARCH="x86_64"; APPIMG_ARCH="amd64" ;;
  aarch64|arm64) DEB_ARCH="arm64"; RPM_ARCH="aarch64"; APPIMG_ARCH="aarch64" ;;
  *) die "Unsupported CPU architecture: $ARCH" ;;
esac

BIN_DIR="${HOME}/.local/bin"
APPS_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"

if [ "$UNINSTALL" -eq 1 ]; then
  say "Removing ${APP}"
  if have dpkg && dpkg -s localsendstudio >/dev/null 2>&1; then $SUDO apt-get remove -y localsendstudio; fi
  if have rpm && rpm -q localsendstudio >/dev/null 2>&1; then $SUDO rpm -e localsendstudio; fi
  rm -f "${BIN_DIR}/${APP}.AppImage" "${APPS_DIR}/localsendstudio.desktop" "${ICON_DIR}/localsendstudio.png"
  say "Done. Your settings stay in ~/.local/share/LocalSendStudio (delete it to remove them)."
  exit 0
fi

have curl || die "curl is required (sudo apt install curl  /  sudo dnf install curl)"

say "Looking up the release"
if [ -n "$VERSION" ]; then
  API="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
else
  API="https://api.github.com/repos/${REPO}/releases/latest"
fi
JSON="$(curl -fsSL -H 'Accept: application/vnd.github+json' "$API")" || die "Could not reach GitHub. Check your internet connection."
TAG="$(printf '%s' "$JSON" | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
[ -n "$TAG" ] || die "Release not found."
URLS="$(printf '%s' "$JSON" | grep '"browser_download_url"' | sed -E 's/.*"(https:[^"]+)".*/\1/')"
pick() { printf '%s\n' "$URLS" | grep -E "$1" | head -1 || true; }

KIND=""
if [ "$FORCE_APPIMAGE" -eq 0 ] && have dpkg && have apt-get; then KIND="deb"
elif [ "$FORCE_APPIMAGE" -eq 0 ] && have rpm && { have dnf || have zypper || have yum; }; then KIND="rpm"
else KIND="appimage"; fi

case "$KIND" in
  deb)      URL="$(pick "_${DEB_ARCH}\.deb$")" ;;
  rpm)      URL="$(pick "\.${RPM_ARCH}\.rpm$")" ;;
  appimage) URL="$(pick "_${APPIMG_ARCH}\.AppImage$")" ;;
esac
if [ -z "${URL:-}" ] && [ "$KIND" != "appimage" ]; then
  warn "No .${KIND} package for ${ARCH} in ${TAG}; falling back to the AppImage."
  KIND="appimage"; URL="$(pick "_${APPIMG_ARCH}\.AppImage$")"
fi
[ -n "${URL:-}" ] || die "This release has no Linux build for ${ARCH}."

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
FILE="${TMP}/$(basename "$URL")"
say "Downloading $(basename "$URL") (${TAG})"
curl -fL --progress-bar -o "$FILE" "$URL"

SUMS_URL="$(pick 'SHA256SUMS\.txt$')"
if [ -n "$SUMS_URL" ] && have sha256sum; then
  curl -fsSL -o "${TMP}/SHA256SUMS.txt" "$SUMS_URL"
  WANT="$(grep -F "$(basename "$FILE")" "${TMP}/SHA256SUMS.txt" | awk '{print $1}' | head -1 || true)"
  if [ -n "$WANT" ]; then
    GOT="$(sha256sum "$FILE" | awk '{print $1}')"
    [ "$WANT" = "$GOT" ] || die "Checksum mismatch - the download is corrupt. Nothing was installed."
    say "Checksum OK"
  else
    warn "No checksum listed for this file; skipping verification."
  fi
else
  warn "No checksum file published for ${TAG}; skipping verification."
fi

case "$KIND" in
  deb)
    say "Installing with apt (needs sudo)"
    $SUDO apt-get install -y "$FILE" ;;
  rpm)
    say "Installing the .rpm (needs sudo)"
    if have dnf; then $SUDO dnf install -y "$FILE"
    elif have zypper; then $SUDO zypper --non-interactive --no-gpg-checks install "$FILE"
    else $SUDO yum install -y "$FILE"; fi ;;
  appimage)
    say "Installing the AppImage to ${BIN_DIR}"
    mkdir -p "$BIN_DIR" "$APPS_DIR" "$ICON_DIR"
    install -m 0755 "$FILE" "${BIN_DIR}/${APP}.AppImage"
    ICON_URL="https://raw.githubusercontent.com/${REPO}/${TAG}/src-tauri/icons/128x128.png"
    curl -fsSL -o "${ICON_DIR}/localsendstudio.png" "$ICON_URL" || warn "Could not fetch the icon (not important)."
    cat > "${APPS_DIR}/localsendstudio.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=LocalSendStudio
Comment=Share files across your local network
Exec=${BIN_DIR}/${APP}.AppImage
Icon=localsendstudio
Categories=Network;Utility;
Terminal=false
DESKTOP
    have update-desktop-database && update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
    if ! have fusermount && ! have fusermount3; then
      warn "AppImages need FUSE: sudo apt install libfuse2   (or: sudo dnf install fuse fuse-libs)"
    fi
    case ":$PATH:" in *":${BIN_DIR}:"*) ;; *) warn "${BIN_DIR} is not in your PATH; start the app from your menu or run ${BIN_DIR}/${APP}.AppImage" ;; esac ;;
esac

# --- firewall -----------------------------------------------------------------
FW_CMDS=""
if have ufw && $SUDO ufw status 2>/dev/null | grep -q "Status: active"; then
  FW_CMDS="for r in 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16; do ufw allow from \$r to any port 47821:47840 proto tcp; ufw allow from \$r to any port 47822 proto udp; done"
elif have firewall-cmd && $SUDO firewall-cmd --state >/dev/null 2>&1; then
  FW_CMDS="firewall-cmd --permanent --add-port=47821-47840/tcp --add-port=47822/udp && firewall-cmd --reload"
fi
if [ -n "$FW_CMDS" ]; then
  if [ "$OPEN_FW" -eq 0 ] && [ "$YES" -eq 0 ] && [ -r /dev/tty ]; then
    printf 'Your firewall is active. Open the ports LocalSendStudio needs (TCP 47821-47840, UDP 47822) for local-network addresses only? [y/N] '
    read -r ans </dev/tty || ans=n
    case "$ans" in y|Y|yes) OPEN_FW=1 ;; esac
  fi
  if [ "$OPEN_FW" -eq 1 ]; then
    say "Opening firewall ports"
    $SUDO sh -c "$FW_CMDS"
  else
    warn "Firewall is active. If other devices cannot see this one, run:  sudo sh -c '${FW_CMDS}'"
  fi
fi

say "${APP} ${TAG} installed. Start it from your applications menu."
