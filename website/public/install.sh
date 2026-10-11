#!/usr/bin/env bash
# Install or update QuickDesk (https://quickdesk.click) on Linux or macOS:
#
#   curl -fsSL https://quickdesk.click/install.sh | bash
#
# Picks the newest release, checks its SHA-256 against the release's
# SHA256SUMS, then installs the .deb (apt), the .rpm (dnf or zypper) or,
# on anything else, the AppImage into ~/.local/bin. Only the package install
# step uses sudo. On a Mac (Apple Silicon) it puts QuickDesk.app in
# /Applications (or ~/Applications), without sudo; downloaded this way it is
# not quarantined, so it opens without Gatekeeper's warning.
# Options (environment variables):
#   QUICKDESK_FORMAT=deb|rpm|appimage   choose the Linux package instead of guessing
#   QUICKDESK_VERSION=0.4.0             a specific release instead of the newest
#   QUICKDESK_DRY_RUN=1                 download and check, but install nothing
set -euo pipefail

# Everything runs from main, so a download cut short runs nothing.
main() {

DL="https://dl.quickdesk.click"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
die() { printf 'quickdesk install: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "this needs '$1'; install it and run again"; }

# macOS has shasum instead of sha256sum.
sum256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d ' ' -f 1
}

need curl
os="$(uname -s)"
case "$os" in
  Linux)
    command -v sha256sum >/dev/null 2>&1 || need sha256sum
    case "$(uname -m)" in
      x86_64 | amd64) ;;
      *) die "QuickDesk has Linux builds for x86_64 only, not $(uname -m)" ;;
    esac
    ;;
  Darwin)
    need shasum
    [ "$(uname -m)" = arm64 ] || die "QuickDesk for macOS needs an Apple Silicon Mac (M1 or newer)"
    ;;
  *) die "QuickDesk runs on Linux and macOS (and Windows: see https://quickdesk.click/download)" ;;
esac

format="${QUICKDESK_FORMAT:-}"
if [ "$os" = Darwin ]; then
  format=macos
elif [ -z "$format" ]; then
  if command -v apt-get >/dev/null 2>&1 && command -v dpkg >/dev/null 2>&1; then
    format=deb
  elif command -v dnf >/dev/null 2>&1 || command -v zypper >/dev/null 2>&1; then
    format=rpm
  else
    format=appimage
  fi
fi
case "$format" in
  deb) pattern='_amd64\.deb$' ;;
  rpm) pattern='\.x86_64\.rpm$' ;;
  appimage) pattern='_amd64\.AppImage$' ;;
  macos) pattern='_aarch64\.app\.tar\.gz$' ;;
  *) die "QUICKDESK_FORMAT must be deb, rpm or appimage, not '$format'" ;;
esac

version="${QUICKDESK_VERSION:-}"
if [ -z "$version" ]; then
  version="$(curl -fsSL "$DL/latest.json" | sed -n 's/^[[:space:]]*"version":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
fi
case "$version" in
  [0-9]*.[0-9]*.[0-9]*) ;;
  *) die "could not find the newest QuickDesk release" ;;
esac

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
# apt reads local packages as its own user; let it.
chmod 755 "$tmp"

say "QuickDesk $version ($format)"
curl -fsSL -o "$tmp/SHA256SUMS" "$DL/releases/$version/SHA256SUMS" || die "release $version not found"
# No match is not an error here: the check below says what is missing.
line="$(grep -E "$pattern" "$tmp/SHA256SUMS" | head -n 1 || true)"
[ -n "$line" ] || die "release $version has no $format package"
want="${line%% *}"
name="${line##* }"
name="${name#\*}"

say "Downloading $name"
curl -fL --progress-bar -o "$tmp/$name" "$DL/releases/$version/$name"
chmod 644 "$tmp/$name"
got="$(sum256 "$tmp/$name")"
[ "$got" = "$want" ] || die "checksum does not match for $name (expected $want, got $got)"
say "Checksum OK"

if [ -n "${QUICKDESK_DRY_RUN:-}" ]; then
  say "Dry run: would install $tmp/$name"
  exit 0
fi

if [ "$format" = macos ]; then
  # /Applications is writable for admin accounts; otherwise the user's own.
  dest=/Applications
  [ -w "$dest" ] || { dest="$HOME/Applications"; mkdir -p "$dest"; }
  mkdir "$tmp/app"
  tar -xzf "$tmp/$name" -C "$tmp/app"
  [ -d "$tmp/app/QuickDesk.app" ] || die "$name does not contain QuickDesk.app"
  # An update: quit the running copy first.
  if pgrep -x QuickDesk >/dev/null 2>&1; then
    say "Quitting the running QuickDesk"
    osascript -e 'quit app "QuickDesk"' >/dev/null 2>&1 || pkill -x QuickDesk || true
    sleep 1
  fi
  rm -rf "$dest/QuickDesk.app"
  mv "$tmp/app/QuickDesk.app" "$dest/QuickDesk.app"
  xattr -dr com.apple.quarantine "$dest/QuickDesk.app" 2>/dev/null || true
  say "Installed $dest/QuickDesk.app"
  open "$dest/QuickDesk.app"
  say "QuickDesk $version is running in the menu bar and updates itself from now on. For auto-paste, allow it under System Settings → Privacy & Security → Accessibility when macOS asks."
  exit 0
fi

sudo_cmd=""
if [ "$format" != appimage ] && [ "$(id -u)" -ne 0 ]; then
  need sudo
  sudo_cmd="sudo"
  say "Installing needs your password (sudo)"
fi

case "$format" in
  deb)
    # stdin is this script when piped into bash: keep the installer off it (sudo asks on the terminal).
    $sudo_cmd apt-get install -y "$tmp/$name" </dev/null
    ;;
  rpm)
    if command -v dnf >/dev/null 2>&1; then
      $sudo_cmd dnf install -y "$tmp/$name" </dev/null
    else
      $sudo_cmd zypper --non-interactive install --allow-unsigned-rpm "$tmp/$name" </dev/null
    fi
    ;;
  appimage)
    dir="$HOME/.local/bin"
    mkdir -p "$dir"
    install -m 0755 "$tmp/$name" "$dir/QuickDesk.AppImage"
    say "Installed $dir/QuickDesk.AppImage (it needs FUSE 2: libfuse2)"
    say "Start it with: $dir/QuickDesk.AppImage"
    exit 0
    ;;
esac

say "QuickDesk $version is installed. Open it from your applications menu; it lives in the top bar and updates itself from now on."
}

main "$@"
