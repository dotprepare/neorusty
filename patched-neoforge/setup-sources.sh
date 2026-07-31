#!/usr/bin/env bash
# Restores upstream NeoForge source files that are gitignored by design.
# Run from the repo root, or from patched-neoforge/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_DIR="$SCRIPT_DIR/.."

# Stamp file written on successful restore. It lives in a gitignored location so
# it is never committed; a tracked file cannot be used as a marker because
# tracked files already exist before upstream sources are restored.
STAMP="$SCRIPT_DIR/.sources-restored"

if [ -f "$STAMP" ]; then
    echo "Upstream sources already restored"
    exit 0
fi

# Ensure the upstream remote-tracking branch is available
if ! git -C "$GIT_DIR" rev-parse --verify upstream/1.21.1 &>/dev/null; then
    echo "Fetching upstream/1.21.1..."
    git -C "$GIT_DIR" remote add upstream https://github.com/neoforged/NeoForge.git 2>/dev/null || true
    git -C "$GIT_DIR" fetch upstream 1.21.1 --depth=1
fi

# Restore trees from upstream: (upstream path, local target, strip-components).
# Each tree is extracted to a temp dir, then only the files missing from the
# working tree are copied in (cp -n). This never overwrites tracked files whose
# line endings have been normalized, and works the same on GNU and BSD tar/cp.
RESTORE_TREES=(
    "src/main/java|$SCRIPT_DIR/src/main/java|3"
    "testframework|$SCRIPT_DIR/testframework|1"
    "coremods|$SCRIPT_DIR/coremods|1"
    "tests|$GIT_DIR/tests|1"
)

TMPDIRS=()
cleanup() {
    for dir in "${TMPDIRS[@]:-}"; do rm -rf "$dir"; done
}
trap cleanup EXIT

# Copy only the files missing from the working tree. This never overwrites
# tracked files whose line endings have been normalized, and works the same on
# macOS and Linux (cp -n and GNU tar's --keep-old-files are not portable).
copy_missing() {
    local tmpdir="$1" target="$2"
    (cd "$tmpdir" && find . -type f -print0) | while IFS= read -r -d '' f; do
        rel="${f#./}"
        if [ ! -e "$target/$rel" ]; then
            mkdir -p "$target/$(dirname "$rel")"
            cp "$tmpdir/$rel" "$target/$rel"
        fi
    done
}

for spec in "${RESTORE_TREES[@]}"; do
    up_path="${spec%%|*}"
    rest="${spec#*|}"
    target="${rest%%|*}"
    strip="${rest##*|}"
    echo "Restoring upstream '$up_path' -> $target"
    mkdir -p "$target"
    tmpdir="$(mktemp -d)"
    TMPDIRS+=("$tmpdir")
    git -C "$GIT_DIR" archive upstream/1.21.1 "$up_path/" | tar x -C "$tmpdir" --strip-components="$strip"
    copy_missing "$tmpdir" "$target"
done

# Remove upstream files overridden by tracked bridge sources
BRIDGE_DIR="$GIT_DIR/neorusty-source/bridge/java/src/main/java"
if [ -d "$BRIDGE_DIR" ]; then
    echo "Removing upstream files overridden by tracked bridge sources..."
    find "$BRIDGE_DIR" -name '*.java' | while read -r bridge_file; do
        rel_path="${bridge_file#$BRIDGE_DIR/}"
        target_file="$SCRIPT_DIR/src/main/java/$rel_path"
        if [ -f "$target_file" ]; then
            rm -f "$target_file"
        fi
    done
fi

COUNT="$(find "$SCRIPT_DIR/src/main/java" "$SCRIPT_DIR/testframework" "$SCRIPT_DIR/coremods" "$GIT_DIR/tests" -name '*.java' 2>/dev/null | wc -l)"
echo "Restored upstream sources ($COUNT java files)"

touch "$STAMP"
