#!/bin/bash
# the app's main program. points the Vulkan loader at the MoltenVK in the
# bundle, which it can't find from anywhere else, and starts Zakuro, which
# opens its library when it is given no game.
here="$(cd "$(dirname "$0")" && pwd)"
export VK_DRIVER_FILES="$here/../Resources/vulkan/icd.d/MoltenVK_icd.json"
exec "$here/zakuro-emulator" "$@"
