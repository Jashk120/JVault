#!/bin/bash
set -e

APP_NAME="JVault"
BINARY_NAME="jvault"
INSTALL_BIN="$HOME/.local/bin"
INSTALL_DESKTOP="$HOME/.local/share/applications"
INSTALL_ICONS="$HOME/.local/share/icons"

# ── Uninstall mode ─────────────────────────────────────────────────────────────
if [[ "$1" == "--uninstall" ]]; then
    echo "Removing $APP_NAME..."
    rm -f "$INSTALL_BIN/$BINARY_NAME"
    rm -f "$INSTALL_DESKTOP/jvault.desktop"
    rm -f "$INSTALL_ICONS/jvault.png"
    command -v update-desktop-database &>/dev/null && \
        update-desktop-database "$INSTALL_DESKTOP"
    echo "$APP_NAME removed."
    exit 0
fi

# ── Install ────────────────────────────────────────────────────────────────────
echo "Installing $APP_NAME..."

mkdir -p "$INSTALL_BIN"
mkdir -p "$INSTALL_DESKTOP"
mkdir -p "$INSTALL_ICONS"

# Binary
cp "$BINARY_NAME" "$INSTALL_BIN/$BINARY_NAME"
chmod +x "$INSTALL_BIN/$BINARY_NAME"

# Icon
cp icon.png "$INSTALL_ICONS/jvault.png"

# .desktop entry
cat > "$INSTALL_DESKTOP/jvault.desktop" << EOF
[Desktop Entry]
Name=JVault
Comment=JVault Application
Exec=$INSTALL_BIN/$BINARY_NAME
Icon=$INSTALL_ICONS/jvault.png
Type=Application
Categories=Game;
Terminal=false
StartupNotify=true
EOF

# Refresh app menu
command -v update-desktop-database &>/dev/null && \
    update-desktop-database "$INSTALL_DESKTOP"

# Make sure ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_BIN:"* ]]; then
    echo ""
    echo "  NOTE: Add this to your ~/.bashrc or ~/.zshrc:"
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
echo "$APP_NAME installed! You can launch it from your app menu or run: $BINARY_NAME"
