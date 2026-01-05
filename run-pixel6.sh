#!/bin/bash
# run-pixel6.sh - Build and run LevitateOS in QEMU with Pixel 6 hardware profile
#
# TEAM_042: Pixel 6 Emulation Profile (Updated with mitigations)
#
# Pixel 6 Tensor SoC:
# - 2x Cortex-X1 @ 2.80 GHz (big)
# - 2x Cortex-A76 @ 2.25 GHz (medium)
# - 4x Cortex-A55 @ 1.80 GHz (little)
# - 8GB LPDDR5
# - GICv3
#
# QEMU Mitigations:
# - cortex-a76: Exact match for medium cores, close to X1
# - Cluster topology: 2 clusters x 4 cores simulates big.LITTLE domains
# - GICv3: Matches Pixel 6
# - 8GB RAM: Matches Pixel 6
#
# Remaining limitation: All cores same type (no true heterogeneous)

set -e

# Configuration (updated with verified QEMU 10.1+ features)
# NOTE: Using GICv2 because kernel GIC driver doesn't support v3 yet
QEMU_MACHINE="virt"
QEMU_CPU="cortex-a76"
QEMU_SMP="8"
QEMU_MEMORY="8G"

echo "=========================================="
echo " LevitateOS - Pixel 6 Emulation Profile"
echo "=========================================="
echo "CPU:    $QEMU_CPU"
echo "SMP:    $QEMU_SMP cores"
echo "RAM:    $QEMU_MEMORY"
echo "GIC:    v2 (v3 TODO - kernel driver needed)"
echo "=========================================="

echo "Building kernel..."
cargo build --release -p levitate-kernel --features verbose --target aarch64-unknown-none

# Path to the compiled ELF
ELF="target/aarch64-unknown-none/release/levitate-kernel"
BIN="kernel64_rust.bin"

echo "Converting to raw binary..."
aarch64-linux-gnu-objcopy -O binary "$ELF" "$BIN"

echo "Launching QEMU (Pixel 6 profile)..."
# Ensure disk image exists
# TEAM_121: Use xtask to ensure disk image is correctly partitioned and populated
cargo xtask build

qemu-system-aarch64 \
    -M "$QEMU_MACHINE" \
    -cpu "$QEMU_CPU" \
    -smp "$QEMU_SMP" \
    -m "$QEMU_MEMORY" \
    -kernel "$BIN" \
    -display gtk,zoom-to-fit=off \
    -device virtio-gpu-pci,xres=2400,yres=1080 \
    -device virtio-keyboard-device \
    -device virtio-tablet-device \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0 \
    -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
    -device virtio-blk-device,drive=hd0 \
    -initrd initramfs.cpio \
    -serial stdio \
    -no-reboot
