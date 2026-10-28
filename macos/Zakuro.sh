#!/bin/bash
# the app's main program. asks for a game when it isn't given one, has
# 3dsrecomp make the game's library the first time, then starts Zakuro.
here="$(cd "$(dirname "$0")" && pwd)"
export VK_DRIVER_FILES="$here/../Resources/vulkan/icd.d/MoltenVK_icd.json"
data="$HOME/Library/Application Support/Zakuro"
log="$HOME/Library/Logs/Zakuro/build.log"
mkdir -p "$data/built" "$(dirname "$log")"

say() { osascript -e "display dialog \"$1\" buttons {\"OK\"} default button 1 with title \"Zakuro\"" >/dev/null; }

rom="$1"
if [ -z "$rom" ]; then
	rom="$(osascript -e 'POSIX path of (choose file with prompt "Choose your Nintendo 3DS game, a decrypted .3ds, .cxi or .cia")')" || exit 0
fi

# the library is made from the game, so it is made here, on the user's own
# Mac, and only once for each game file
key="$(printf '%s' "$rom$(stat -f '%z %m' "$rom")" | shasum | cut -d' ' -f1)"
if [ ! -e "$data/built/$key" ]; then
	if ! xcrun --find clang >/dev/null 2>&1; then
		say "Zakuro needs Apple's command line tools to prepare a game. Run xcode-select --install in Terminal, then open Zakuro again."
		exit 1
	fi
	osascript -e 'display notification "The first start of a game takes several minutes." with title "Zakuro"'
	if "$here/3dsrecomp" build "$rom" >"$log" 2>&1; then
		touch "$data/built/$key"
	else
		say "Preparing the game failed. The log is at $log"
		exit 1
	fi
fi

exec "$here/zakuro-emulator" "$rom"
