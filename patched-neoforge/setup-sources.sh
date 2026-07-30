#!/usr/bin/env bash
# Restores upstream NeoForge source files (gitignored by design).
# Run from the repo root, or from patched-neoforge/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_DIR="$SCRIPT_DIR/.."
TARGET="$SCRIPT_DIR/src/main/java"

# Pick a file that exists in upstream/1.21.1 but NOT in our tracked sources
NEEDLE="$TARGET/net/neoforged/neoforge/common/NeoForge.java"

if [ -f "$NEEDLE" ]; then
    echo "Upstream sources already present ($(find "$TARGET" -name '*.java' | wc -l) files)"
    exit 0
fi

# Ensure the upstream remote-tracking branch is available
if ! git -C "$GIT_DIR" rev-parse --verify upstream/1.21.1 &>/dev/null; then
    echo "Fetching upstream/1.21.1..."
    git -C "$GIT_DIR" remote add upstream https://github.com/neoforged/NeoForge.git 2>/dev/null || true
    git -C "$GIT_DIR" fetch upstream 1.21.1 --depth=1
fi

echo "Restoring NeoForge source files from upstream/1.21.1..."
mkdir -p "$TARGET"

git -C "$GIT_DIR" archive upstream/1.21.1 src/main/java/ | tar x -C "$TARGET" --strip-components=3

BRIDGE_DIR="$SCRIPT_DIR/../neorusty-source/bridge/java/src/main/java"
if [ -d "$BRIDGE_DIR" ]; then
    echo "Removing upstream files overridden by tracked bridge sources..."
    find "$BRIDGE_DIR" -name '*.java' | while read -r bridge_file; do
        rel_path="${bridge_file#$BRIDGE_DIR/}"
        target_file="$TARGET/$rel_path"
        if [ -f "$target_file" ]; then
            rm -f "$target_file"
        fi
    done
fi

COUNT="$(find "$TARGET" -name '*.java' 2>/dev/null | wc -l)"
echo "Restored $COUNT non-overridden upstream source files"
