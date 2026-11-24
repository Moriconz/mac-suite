#!/bin/bash
# Uso: scripts/bundle.sh <app> [Nome visualizzato]
# Variabili opzionali:
#   SIGN_ID="Developer ID Application: ..."  firma per la distribuzione (default: firma ad-hoc, solo uso locale)
#   NOTARY_PROFILE=<profilo>                 notarizza e applica lo staple (crea il profilo con: xcrun notarytool store-credentials)
#   ENTITLEMENTS=path.plist                  entitlement dell'app
set -euo pipefail
cd "$(dirname "$0")/.."

BIN=${1:?nome del crate in apps/}
NAME=${2:-$BIN}
VERSION=$(cargo pkgid -p "$BIN" | sed 's/.*[#@]//')
ID="com.macsuite.$BIN"
APP="target/release/$NAME.app"

cargo build --release -p "$BIN"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "target/release/$BIN" "$APP/Contents/MacOS/$BIN"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>$NAME</string>
  <key>CFBundleDisplayName</key><string>$NAME</string>
  <key>CFBundleIdentifier</key><string>$ID</string>
  <key>CFBundleExecutable</key><string>$BIN</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleDevelopmentRegion</key><string>it</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
EOF

SIGN_ARGS=(--force --options runtime --sign "${SIGN_ID:--}")
[[ -n "${SIGN_ID:-}" ]] && SIGN_ARGS+=(--timestamp)
[[ -n "${ENTITLEMENTS:-}" ]] && SIGN_ARGS+=(--entitlements "$ENTITLEMENTS")
codesign "${SIGN_ARGS[@]}" "$APP"
codesign --verify --strict "$APP"

if [[ -n "${NOTARY_PROFILE:-}" ]]; then
  [[ -n "${SIGN_ID:-}" ]] || { echo "La notarizzazione richiede SIGN_ID (Developer ID)"; exit 1; }
  ZIP="target/release/$NAME.zip"
  ditto -c -k --keepParent "$APP" "$ZIP"
  xcrun notarytool submit "$ZIP" --keychain-profile "$NOTARY_PROFILE" --wait
  xcrun stapler staple "$APP"
  spctl --assess --type execute --verbose "$APP"
fi

echo "OK: $APP"
