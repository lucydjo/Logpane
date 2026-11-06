#!/usr/bin/env bash
# Build an unsigned local Logpane.app bundle for Apple Silicon.
set -euo pipefail

cd "$(dirname "$0")/.."

APP_NAME="Logpane"
BUNDLE="${APP_NAME}.app"
BIN="target/release/logpane"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"

echo "==> cargo build --release (v${VERSION})"
cargo build --release --locked

echo "==> assembling ${BUNDLE}"
rm -rf "${BUNDLE}"
mkdir -p "${BUNDLE}/Contents/MacOS" "${BUNDLE}/Contents/Resources"
cp "${BIN}" "${BUNDLE}/Contents/MacOS/${APP_NAME}"

cat > "${BUNDLE}/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>Logpane</string>
    <key>CFBundleIdentifier</key>
    <string>io.github.lucydjo.logpane</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Logpane</string>
    <key>CFBundleDisplayName</key>
    <string>Logpane</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>__VERSION__</string>
    <key>CFBundleVersion</key>
    <string>__VERSION__</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Logpane</string>
</dict>
</plist>
PLIST

sed -i "" "s/__VERSION__/${VERSION}/g" "${BUNDLE}/Contents/Info.plist"

# Ad-hoc signature: enough for local use, replace with a Developer ID for distribution.
if command -v codesign >/dev/null 2>&1; then
    codesign --force --deep --sign - "${BUNDLE}" >/dev/null 2>&1 || {
        echo "warning: ad-hoc codesign failed; the bundle still runs locally" >&2
    }
fi

echo "==> built ${BUNDLE}"
echo "    open ${BUNDLE}"
