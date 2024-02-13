#!/usr/bin/env bash

# Simply cross-compiles this project for aarch64 on musl based distros (openwrt).
command -v rustup &> /dev/null || { echo "This script requires 'rustup'"; exit 0; }
command -v cargo &> /dev/null  || { echo "This script requires 'cargo'"; exit 0; }

# Install target if not already installed.
TARGET_NAME="aarch64-unknown-linux-musl"
rustup target list --installed | grep -s "$TARGET_NAME" || rustup target install "$TARGET_NAME"

# Build it!
echo "Building release for '$TARGET_NAME'"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld
cargo build --release --target "$TARGET_NAME"
