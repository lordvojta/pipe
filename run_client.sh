#!/bin/bash
# Convenience wrapper for running the Python client

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

source venv/bin/activate
python client.py "$@"
