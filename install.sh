#!/bin/bash
# Installation script for terminal-to-ps

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Installing terminal-to-ps ==="
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Install it from https://rustup.rs/"
    exit 1
fi

# Check for Python3
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is not installed."
    exit 1
fi

# Build Rust server
echo "Building Rust server..."
cargo build --release
echo "✓ Server built successfully"
echo ""

# Set up Python virtual environment
if [ ! -d "venv" ]; then
    echo "Creating Python virtual environment..."
    python3 -m venv venv
fi

echo "Installing Python dependencies..."
source venv/bin/activate
pip install --upgrade pip > /dev/null
pip install cryptography > /dev/null
if [ "$(uname)" = "Darwin" ] || [ "$(uname)" = "Linux" ]; then
    echo "Unix-like system detected, skipping pywin32"
else
    pip install pywin32 > /dev/null
fi
echo "✓ Python dependencies installed"
echo ""

# Make scripts executable
chmod +x run_server.sh
chmod +x run_client.sh

echo "=== Installation complete! ==="
echo ""
echo "Quick start:"
echo "  1. Start server: ./run_server.sh"
echo "  2. In another terminal, send data: ./run_client.sh send \"Hello World\""
echo ""
echo "For auto-start on macOS, run:"
echo "  ./setup_autostart.sh"
echo ""
