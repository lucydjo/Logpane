#!/usr/bin/env bash
# Regenerate docs/screenshots/ and docs/demo.gif from synthetic demo notes.
# Opens the debug build a few times; each run closes itself after its capture.
set -euo pipefail

cd "$(dirname "$0")/.."

OUT="docs/screenshots"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

cargo build --locked
cargo run --quiet --locked --example seed_demo -- "${WORK}/demo"
mkdir -p "${WORK}/blank" "${OUT}"

shot() { # <name> <data dir> <demo state>
    echo "==> ${1}"
    LOGPANE_DATA_DIR="${2}" LOGPANE_DEMO="${3}" LOGPANE_SCREENSHOT="${OUT}/${1}.png" \
        ./target/debug/logpane --show
}

shot empty  "${WORK}/blank" "theme:dark"
shot dark   "${WORK}/demo"  "theme:dark"
shot search "${WORK}/demo"  "theme:dark,search:latency"
shot help   "${WORK}/demo"  "theme:dark,help"
shot light  "${WORK}/demo"  "theme:light"

if command -v ffmpeg >/dev/null 2>&1; then
    echo "==> demo.gif"
    LIST="${WORK}/frames.txt"
    : > "${LIST}"
    for frame in empty dark search help light; do
        printf "file '%s'\nduration 2.2\n" "$(pwd)/${OUT}/${frame}.png" >> "${LIST}"
    done
    printf "file '%s'\n" "$(pwd)/${OUT}/light.png" >> "${LIST}"
    ffmpeg -loglevel error -y -f concat -safe 0 -i "${LIST}" \
        -vf "scale=1200:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse=dither=none" \
        docs/demo.gif
fi
echo "done"
