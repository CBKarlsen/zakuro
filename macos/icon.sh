#!/bin/bash
# makes macos/Zakuro.icns from icon.swift. it is kept in the repository, so
# this only needs running when the drawing changes. usage: macos/icon.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
iconset="$work/Zakuro.iconset"
mkdir "$iconset"
swift "$here/icon.swift" "$work/icon.png"
for side in 16 32 128 256 512; do
    sips -z $side $side "$work/icon.png" --out "$iconset/icon_${side}x${side}.png" >/dev/null
    sips -z $((side * 2)) $((side * 2)) "$work/icon.png" --out "$iconset/icon_${side}x${side}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$here/Zakuro.icns"
rm -rf "$work"
