#!/usr/bin/env bash
# Prepares the pinned Pumpkin 1.21.1 submodule for the neorusty build.
# Run from anywhere; assumes the submodule is checked out at the pinned commit.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PUMPKIN_ROOT="$SCRIPT_DIR/../../pumpkin-master"
PUMPKIN_SRC="$PUMPKIN_ROOT/pumpkin/src"
BACKPORTS_DIR="$SCRIPT_DIR/backports"

if [ ! -d "$PUMPKIN_SRC" ]; then
    echo "error: $PUMPKIN_ROOT is not checked out (run 'git submodule update --init')" >&2
    exit 1
fi

echo "Injecting embeddable lib.rs into Pumpkin 1.21.1..."
cp "$SCRIPT_DIR/pumpkin-1.21.1-lib.rs" "$PUMPKIN_SRC/lib.rs"
rm -f "$PUMPKIN_SRC/main.rs"

if [ -d "$BACKPORTS_DIR" ]; then
    for patch in "$BACKPORTS_DIR"/*.patch; do
        [ -e "$patch" ] || continue
        echo "Applying backport patch: $(basename "$patch")"
        git -C "$PUMPKIN_ROOT" apply --check "$patch"
        git -C "$PUMPKIN_ROOT" apply "$patch"
    done
fi

echo "Pumpkin 1.21.1 patched."
