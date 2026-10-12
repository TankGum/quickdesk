#!/usr/bin/env bash
# CI (macOS): make QuickDesk's own code signing certificate available to
# `tauri build`. It is self-signed, not Apple's: Gatekeeper treats the app as
# it treats an ad hoc one, but macOS then knows QuickDesk by that certificate
# instead of by each build's hash, so the Keychain's "Always Allow" and the
# Accessibility permission carry over to every update. See docs/RELEASING.md.
#
# Needs MACOS_CERTIFICATE (the .p12, base64) and MACOS_CERTIFICATE_PASSWORD.
# Without them the build stays signed ad hoc.
set -euo pipefail

if [ -z "${MACOS_CERTIFICATE:-}" ]; then
  echo "::warning::MACOS_CERTIFICATE is not set: the app is signed ad hoc, and macOS asks for the Keychain and Accessibility again after each update"
  exit 0
fi

keychain="$RUNNER_TEMP/quickdesk-signing.keychain-db"
p12="$RUNNER_TEMP/quickdesk-signing.p12"
password=$(openssl rand -hex 16)

echo "$MACOS_CERTIFICATE" | base64 --decode > "$p12"
security create-keychain -p "$password" "$keychain"
security set-keychain-settings -lut 3600 "$keychain"
security unlock-keychain -p "$password" "$keychain"
security import "$p12" -k "$keychain" -P "$MACOS_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
rm -f "$p12"
# Let codesign use the key without a dialog.
security set-key-partition-list -S apple-tool:,apple: -s -k "$password" "$keychain" > /dev/null
# codesign finds the identity only in a keychain on the search list.
# shellcheck disable=SC2046
security list-keychains -d user -s "$keychain" $(security list-keychains -d user | xargs)

# The certificate is not trusted by the system, so it is picked by its SHA-1
# (codesign does not find an untrusted identity by its name).
identity=$(security find-identity -p codesigning "$keychain" | awk '/[0-9A-F]{40}/ { print $2; exit }')
if [ -z "$identity" ]; then
  echo "::error::no code signing identity in MACOS_CERTIFICATE"
  exit 1
fi
echo "APPLE_SIGNING_IDENTITY=$identity" >> "$GITHUB_ENV"
echo "signing with $identity"
