#!/bin/bash
# makes Zakuro.app from a release build of Zakuro, with MoltenVK and the
# Vulkan loader copied in from Homebrew's, so the app runs on a Mac that has
# none of them. usage: macos/bundle.sh
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
brew="$(brew --prefix)"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)"
app="$root/target/release/Zakuro.app"

cargo build --release -p zakuro --manifest-path "$root/Cargo.toml"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Frameworks" "$app/Contents/Resources/vulkan/icd.d"
sed "s/VERSION/$version/" "$root/macos/Info.plist" >"$app/Contents/Info.plist"
install -m 644 "$root/macos/Zakuro.icns" "$app/Contents/Resources/Zakuro.icns"
install -m 755 "$root/macos/Zakuro.sh" "$app/Contents/MacOS/Zakuro"
install -m 755 "$root/target/release/zakuro" "$app/Contents/MacOS/zakuro-emulator"
install -m 644 "$brew/lib/libvulkan.1.dylib" "$app/Contents/Frameworks/libvulkan.dylib"
install -m 644 "$brew/opt/molten-vk/lib/libMoltenVK.dylib" "$app/Contents/Frameworks/libMoltenVK.dylib"
# the driver file is found from the loader's folder, so the path goes from there
cat >"$app/Contents/Resources/vulkan/icd.d/MoltenVK_icd.json" <<'JSON'
{
    "file_format_version" : "1.0.0",
    "ICD": {
        "library_path": "../../../Frameworks/libMoltenVK.dylib",
        "api_version" : "1.4.0",
        "is_portability_driver" : true
    }
}
JSON
# the copies are changed, so they are signed again, ad hoc
codesign --force --deep --sign - "$app"
echo "$app"
