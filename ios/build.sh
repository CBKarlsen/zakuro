#!/bin/bash
# makes Zakuro.app for the iOS Simulator, one game's recompiled code linked in.
# usage: ios/build.sh <program id> <folder with the game's librecomp.a> <MoltenVK library>
#
# the code has to be compiled for the simulator, which 3dsrecomp port does
# when CC names the simulator's compiler:
#   CC="xcrun --sdk iphonesimulator clang -arch arm64 -mios-simulator-version-min=17.0" 3dsrecomp port <rom> <folder>
#
# the app is only signed ad hoc, which the simulator takes and a device doesn't.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
program_id="$1"
code="$2"
moltenvk="$3"
target=aarch64-apple-ios-sim
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)"
app="$root/target/ios/Zakuro.app"

cargo build --release --target "$target" -p zakuro-apple --manifest-path "$root/Cargo.toml"
rm -rf "$app"
mkdir -p "$app/Frameworks"
# the native libraries are the ones cargo rustc --print native-static-libs names for zakuro-apple
xcrun --sdk iphonesimulator swiftc -target arm64-apple-ios17.0-simulator -O -parse-as-library \
    -import-objc-header "$root/ios/Zakuro-Bridging-Header.h" "$root/ios/ZakuroApp.swift" \
    -L "$root/target/$target/release" -L "$code" -lzakuro_apple -lrecomp \
    -framework Foundation -framework CoreGraphics -framework CoreFoundation -lobjc -liconv -lm \
    -o "$app/Zakuro"
sed "s/VERSION/$version/; s/PROGRAMID/$program_id/" "$root/ios/Info.plist" >"$app/Info.plist"
install -m 644 "$moltenvk" "$app/Frameworks/libMoltenVK.dylib"
codesign --force --sign - "$app/Frameworks/libMoltenVK.dylib"
codesign --force --sign - "$app"
echo "$app"
