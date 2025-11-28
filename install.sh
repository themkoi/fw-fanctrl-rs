#!/bin/bash
set -e

# Compile Rust project in release mode
cargo build --release

# Get the binary name from Cargo.toml
BINARY_NAME=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].targets[] | select(.kind[]=="bin") | .name')

# Copy the binary to /usr/local/bin (requires sudo)
sudo cp "target/release/$BINARY_NAME" /usr/local/bin/

echo "Installed $BINARY_NAME to /usr/local/bin"
