#!/bin/bash

echo "╔════════════════════════════════════════════════════════════╗"
echo "║          FALLOUT: WASTELAND ADVENTURES                     ║"
echo "║          Building and launching...                         ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust is not installed!"
    echo "Please install from: https://rustup.rs/"
    echo ""
    exit 1
fi

# Create saves directory if it doesn't exist
mkdir -p saves

# Build and run
echo "Building game..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "Build successful! Starting game..."
    echo ""
    cargo run --release
else
    echo ""
    echo "Build failed! Check errors above."
    exit 1
fi
