#!/usr/bin/env bash
# Install or update QuickDesk (https://quickdesk.click) on Linux:
#
#   curl -fsSL https://quickdesk.click/install.sh | bash
#
# Picks the newest release, checks its SHA-256 against the release's
# SHA256SUMS, then installs the .deb (apt), the .rpm (dnf or zypper) or,
# on anything else, the AppImage into ~/.local/bin. Only the package install
# step uses sudo. Options (environment variables):
#   QUICKDESK_FORMAT=deb|rpm|appimage   choose the package instead of guessing
#   QUICKDESK_VERSION=0.4.0             a specific release instead of the newest
#   QUICKDESK_DRY_RUN=1                 download and check, but install nothing
set -euo pipefail

# Everything runs from main, so a download cut short runs nothing.
main() {

DL="https://dl.quickdesk.click"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
die() { printf 'quickdesk install: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "this needs '$1'; install it and run again"; }

need curl
need sha256sum
[ "$(uname -s)" = Linux ] || die "QuickDesk runs on Linux only for now"
case "$(uname -m)" in
  x86_64 | amd64) ;;
  *) die "QuickDesk has builds for x86_64 only, not $(uname -m)" ;;
esac

format="${QUICKDESK_FORMAT:-}"
if [ -z "$format" ]; then
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
line="$(grep -E "$pattern" "$tmp/SHA256SUMS" | head -n 1)"
[ -n "$line" ] || die "release $version has no $format package"
want="${line%% *}"
name="${line##* }"
name="${name#\*}"

say "Downloading $name"
curl -fL --progress-bar -o "$tmp/$name" "$DL/releases/$version/$name"
chmod 644 "$tmp/$name"
got="$(sha256sum "$tmp/$name" | cut -d ' ' -f 1)"
[ "$got" = "$want" ] || die "checksum does not match for $name (expected $want, got $got)"
say "Checksum OK"

if [ -n "${QUICKDESK_DRY_RUN:-}" ]; then
  say "Dry run: would install $tmp/$name"
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
