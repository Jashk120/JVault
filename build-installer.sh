#!/bin/bash
set -e

PROJECT="$HOME/JVault"
RELEASE_DIR="$PROJECT/release"
BINARY="$PROJECT/src-tauri/target/release/jvault"
ICON="$PROJECT/src-tauri/icons/128x128.png"
INSTALL_SCRIPT="$PROJECT/install.sh"
OUTPUT="JVault-installer.run"

# ── 1. Build ───────────────────────────────────────────────────────────────────
echo "Building JVault..."
cd "$PROJECT/src-tauri"
cargo tauri build

# ── 2. Stage release dir ───────────────────────────────────────────────────────
echo "Staging release..."
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

cp "$BINARY"         "$RELEASE_DIR/jvault"
cp "$ICON"           "$RELEASE_DIR/icon.png"
cp "$INSTALL_SCRIPT" "$RELEASE_DIR/install.sh"
chmod +x "$RELEASE_DIR/install.sh"

# ── 3. Bundle with makeself ────────────────────────────────────────────────────
echo "Bundling installer..."
cd "$PROJECT"
makeself \
    --version "1.0.0" \
    "$RELEASE_DIR" \
    "$OUTPUT" \
    "JVault Installer" \
    ./install.sh

echo ""
echo "Done! Installer: $PROJECT/$OUTPUT"
echo "Run with:  chmod +x $OUTPUT && ./$OUTPUT"
echo "Uninstall: ./$OUTPUT -- --uninstall"
