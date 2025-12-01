#!/bin/bash
# Convenience wrapper for running the Rust server

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Starting terminal-to-ps server..."
echo "Press Ctrl+C to stop"
echo ""

# Build and run in release mode for better performance
cargo run --release
