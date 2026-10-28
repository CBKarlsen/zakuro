#!/bin/bash
# makes Zakuro.app for the iOS Simulator or for an iPhone, one game's
# recompiled code linked in.
# usage: ios/build.sh <simulator|device> <program id> <folder with the game's librecomp.a> <MoltenVK library>
#
# the code has to be compiled for where the app goes, which 3dsrecomp port
# does when CC names the compiler of that SDK, for the simulator
#   CC="xcrun --sdk iphonesimulator clang -arch arm64 -mios-simulator-version-min=17.0"
# and for a device
#   CC="xcrun --sdk iphoneos clang -arch arm64 -miphoneos-version-min=17.0"
# as in CC=... 3dsrecomp port <rom> <folder>. MoltenVK is the library of the
# same platform, from its Package/Release/MoltenVK/dynamic.
#
# the simulator takes an app signed ad hoc. a device wants a profile that
# lists it, and a certificate the profile lists:
#   PROFILE=<file.mobileprovision> SIGN_IDENTITY=<name or hash of the certificate> ios/build.sh device ...
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
platform="$1"
program_id="$2"
code="$3"
moltenvk="$4"
case "$platform" in
    simulator) target=aarch64-apple-ios-sim; sdk=iphonesimulator; swift_target=arm64-apple-ios17.0-simulator ;;
    device) target=aarch64-apple-ios; sdk=iphoneos; swift_target=arm64-apple-ios17.0 ;;
    *) echo "the platform is simulator or device" >&2; exit 2 ;;
esac
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)"
app="$root/target/ios/$platform/Zakuro.app"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

cargo build --release --target "$target" -p zakuro-apple --manifest-path "$root/Cargo.toml"
rm -rf "$app"
mkdir -p "$app/Frameworks"
# the native libraries are the ones cargo rustc --print native-static-libs names for zakuro-apple
xcrun --sdk "$sdk" swiftc -target "$swift_target" -O -parse-as-library \
    -import-objc-header "$root/ios/Zakuro-Bridging-Header.h" "$root/ios/ZakuroApp.swift" \
    -L "$root/target/$target/release" -L "$code" -lzakuro_apple -lrecomp \
    -framework Foundation -framework CoreGraphics -framework CoreFoundation -framework AudioToolbox -framework CoreAudio -lobjc -liconv -lm \
    -o "$app/Zakuro"
sed "s/VERSION/$version/; s/PROGRAMID/$program_id/" "$root/ios/Info.plist" >"$app/Info.plist"
install -m 644 "$moltenvk" "$app/Frameworks/libMoltenVK.dylib"

if [ "$platform" = simulator ]; then
    codesign --force --sign - "$app/Frameworks/libMoltenVK.dylib"
    codesign --force --sign - "$app"
else
    : "${PROFILE:?a device needs PROFILE, the provisioning profile that lists it}"
    : "${SIGN_IDENTITY:?a device needs SIGN_IDENTITY, the certificate the profile lists}"
    security cms -D -i "$PROFILE" >"$work/profile.plist"
    team="$(plutil -extract TeamIdentifier.0 raw "$work/profile.plist")"
    bundle="$(plutil -extract CFBundleIdentifier raw "$app/Info.plist")"
    cat >"$work/entitlements.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
	<key>application-identifier</key><string>$team.$bundle</string>
	<key>com.apple.developer.team-identifier</key><string>$team</string>
	<key>get-task-allow</key><true/>
</dict></plist>
PLIST
    cp "$PROFILE" "$app/embedded.mobileprovision"
    codesign --force --sign "$SIGN_IDENTITY" "$app/Frameworks/libMoltenVK.dylib"
    codesign --force --sign "$SIGN_IDENTITY" --entitlements "$work/entitlements.plist" "$app"
    codesign --verify --strict "$app"
fi
echo "$app"
