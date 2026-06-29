#!/bin/bash
set -e

# Build optimized for the current CPU
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Get the binary name from Cargo.toml
BINARY_NAME=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].targets[] | select(.kind[]=="bin") | .name')

# Install the binary
sudo install -Dm755 "target/release/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME"

echo "Installed $BINARY_NAME to /usr/local/bin"
