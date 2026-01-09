#!/bin/bash
# TEAM_351: Build Eyra test binary for LevitateOS
# Uses rust-toolchain.toml in eyra-hello for correct nightly version
set -e

ARCH=${1:-aarch64}

case $ARCH in
    aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    x86_64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    *)
        echo "Usage: $0 [aarch64|x86_64]"
        exit 1
        ;;
esac

echo "=== Building eyra-hello for $ARCH ==="

cd "$(dirname "$0")/../userspace/eyra-hello"

# Build with -Zbuild-std (uses rust-toolchain.toml for nightly version)
cargo build \
    --release \
    --target "$TARGET" \
    -Zbuild-std=std,panic_abort

BINARY="target/$TARGET/release/eyra-hello"

if [ -f "$BINARY" ]; then
    echo "=== Build successful ==="
    file "$BINARY"
    ls -lh "$BINARY"
    
    # Copy to initramfs staging area
    INITRAMFS_DIR="../../initramfs"
    if [ -d "$INITRAMFS_DIR" ]; then
        cp "$BINARY" "$INITRAMFS_DIR/"
        echo "Copied to $INITRAMFS_DIR/"
    fi
else
    echo "=== Build failed ==="
    exit 1
fi
