#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Building Java agent JAR ==="
JAVA_SRC="$ROOT/bridge/java/src/main/java"
CLASSES="$ROOT/bridge/java/build/classes"
mkdir -p "$CLASSES"
/usr/bin/javac -d "$CLASSES" "$JAVA_SRC"/neorusty/agent/*.java
/usr/bin/jar cf "$ROOT/bridge/java/build/neorusty-agent.jar" -C "$CLASSES" .

echo "=== Building Rust workspace ==="
/usr/bin/env PATH="/bin:/usr/bin:/usr/local/bin:/opt/homebrew/bin:$HOME/.cargo/bin" \
  cargo build --workspace

echo ""
echo "Done. Artifacts:"
echo "  JAR:       bridge/java/build/neorusty-agent.jar"
echo "  CLI:       target/debug/neorusty-cli"
echo "  Installer: target/debug/neorusty-installer"
echo ""
echo "Install locally:  cp target/debug/neorusty-cli ~/.cargo/bin/neorusty"
echo "Run:              neorusty run"
