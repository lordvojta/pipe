#!/bin/bash
# Set up automatic startup for terminal-to-ps server on macOS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLIST_NAME="com.terminal-to-ps.plist"
PLIST_SRC="$SCRIPT_DIR/$PLIST_NAME"
PLIST_DEST="$HOME/Library/LaunchAgents/$PLIST_NAME"

echo "=== Setting up auto-start for terminal-to-ps ==="
echo ""

# Check if running on macOS
if [ "$(uname)" != "Darwin" ]; then
    echo "This script is for macOS only."
    echo "For Linux, you'll need to create a systemd service."
    exit 1
fi

# Make sure the binary is built
if [ ! -f "$SCRIPT_DIR/target/release/terminal-to-ps" ]; then
    echo "Server binary not found. Building..."
    cd "$SCRIPT_DIR"
    cargo build --release
fi

# Create LaunchAgents directory if it doesn't exist
mkdir -p "$HOME/Library/LaunchAgents"

# Create plist from template
echo "Creating launch agent configuration..."
sed "s|REPLACE_WITH_PATH|$SCRIPT_DIR|g" "$PLIST_SRC" > "$PLIST_DEST"

# Unload if already loaded
launchctl unload "$PLIST_DEST" 2>/dev/null || true

# Load the launch agent
echo "Loading launch agent..."
launchctl load "$PLIST_DEST"

echo ""
echo "✓ Auto-start configured successfully!"
echo ""
echo "The server will now:"
echo "  - Start automatically when you log in"
echo "  - Restart automatically if it crashes"
echo ""
echo "Useful commands:"
echo "  Check status: launchctl list | grep terminal-to-ps"
echo "  Stop server:  launchctl stop $PLIST_NAME"
echo "  Disable auto-start: launchctl unload \"$PLIST_DEST\""
echo "  View logs:    tail -f \"$SCRIPT_DIR/server.log\""
echo ""
