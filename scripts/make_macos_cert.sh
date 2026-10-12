#!/usr/bin/env bash
# Once: create QuickDesk's macOS code signing certificate (self-signed, 20
# years) for scripts/macos_signing.sh. Writes ~/.tauri/quickdesk-macos.p12 and
# its password next to it, and copies the base64 for the GitHub secret.
# Back both files up: a new certificate makes macOS ask once more for the
# Keychain and Accessibility on every Mac. See docs/RELEASING.md.
set -euo pipefail

out="$HOME/.tauri/quickdesk-macos.p12"
if [ -e "$out" ]; then
  echo "$out already exists; not replacing it." >&2
  exit 1
fi
mkdir -p "$HOME/.tauri"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
umask 077

cat > "$work/cfg" <<'EOF'
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = QuickDesk
O = QuickDesk
[ext]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
EOF
openssl req -x509 -newkey rsa:3072 -nodes -keyout "$work/key.pem" -out "$work/cert.pem" \
  -days 7300 -config "$work/cfg" 2> /dev/null
password=$(openssl rand -hex 24)
# -legacy: macOS's `security import` cannot read OpenSSL 3's default PKCS#12.
legacy=$(openssl version | grep -q '^OpenSSL 3' && echo -legacy || true)
openssl pkcs12 -export $legacy -inkey "$work/key.pem" -in "$work/cert.pem" -name QuickDesk \
  -out "$out" -passout "pass:$password"
printf %s "$password" > "$out.password"

openssl x509 -in "$work/cert.pem" -noout -subject -enddate -fingerprint -sha1
base64 -i "$out" | pbcopy
cat <<EOF

Created $out (+ .password). Now, in GitHub → Settings → Secrets and variables → Actions:
  MACOS_CERTIFICATE           paste (the base64 is on the clipboard)
  MACOS_CERTIFICATE_PASSWORD  the contents of $out.password
EOF
