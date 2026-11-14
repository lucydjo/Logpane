#!/usr/bin/env bash
# Rebuild resources/Logpane.icns and docs/icon.png from resources/icon.svg (macOS tools only).
set -euo pipefail

cd "$(dirname "$0")/.."

WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT
SET="${WORK}/Logpane.iconset"
mkdir -p "${SET}"

sips -s format png resources/icon.svg --out "${WORK}/1024.png" >/dev/null
for size in 16 32 128 256 512; do
    sips -z "${size}" "${size}" "${WORK}/1024.png" --out "${SET}/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z "${double}" "${double}" "${WORK}/1024.png" --out "${SET}/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "${SET}" -o resources/Logpane.icns
sips -z 256 256 "${WORK}/1024.png" --out docs/icon.png >/dev/null
echo "built resources/Logpane.icns and docs/icon.png"
