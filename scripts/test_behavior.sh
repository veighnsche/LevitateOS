#!/bin/bash
# scripts/test_behavior.sh
# Verifies that the kernel boot output matches the "Golden Log".

set -e

# Base directories
PROJECT_ROOT="$(dirname "$0")/.."
cd "$PROJECT_ROOT"
TEST_DIR="tests"
GOLDEN_FILE="$TEST_DIR/golden_boot.txt"
ACTUAL_FILE="$TEST_DIR/actual_boot.txt"
KERNEL_BIN="kernel64_rust.bin"

# Ensure clean state
pkill -f qemu-system-aarch64 || true
rm -f "$ACTUAL_FILE"
mkdir -p "$TEST_DIR"

echo "Building kernel..."
cargo build --release --quiet
aarch64-linux-gnu-objcopy -O binary target/aarch64-unknown-none/release/levitate-kernel "$KERNEL_BIN"

echo "Running QEMU (Headless)..."
# Run QEMU for 5 seconds, capture serial output to ACTUAL_FILE
timeout 5s qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a53 \
    -m 512M \
    -kernel "$KERNEL_BIN" \
    -display none \
    -serial file:"$ACTUAL_FILE" \
    -device virtio-gpu-device \
    -device virtio-keyboard-device \
    -device virtio-tablet-device \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0 \
    -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
    -device virtio-blk-device,drive=hd0 \
    -no-reboot || true

# Check if QEMU produced output
if [ ! -f "$ACTUAL_FILE" ]; then
    echo "❌ Error: No output file generated."
    exit 1
fi

# Compare against Golden
echo "Comparing output against Golden Log..."
if diff -u --strip-trailing-cr "$GOLDEN_FILE" "$ACTUAL_FILE"; then
    echo "✅ SUCCESS: Current behavior matches Golden Log."
    exit 0
else
    echo "❌ FAILURE: Behavior REGRESSION detected!"
    echo "Diff shown above."
    # echo "Actual output:"
    # cat "$ACTUAL_FILE"
    exit 1
fi
