#!/bin/bash
# run.sh - Build and run ClaudeOS Rust in QEMU

# Exit on any error
set -e

echo "Building ClaudeOS Rust kernel..."
cargo build --release

# Path to the compiled ELF
ELF="target/aarch64-unknown-none/release/levitate-kernel"
BIN="kernel64_rust.bin"

echo "Converting to raw binary..."
aarch64-linux-gnu-objcopy -O binary "$ELF" "$BIN"

echo "Launching QEMU..."
# Match the parameters from the C run.sh as much as possible
# -M virt: use the 'virt' board
# -cpu cortex-a53: ARMv8-A CPU
# -m 512M: 512MB RAM
# -kernel: the raw binary to load
# -serial mon:stdio: multiplex QEMU monitor and guest serial on stdio
# -display none: hide graphical window for now (until Phase 3)
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a53 \
    -m 512M \
    -kernel kernel64_rust.bin \
    -display none \
    -device virtio-gpu-device \
    -device virtio-keyboard-device \
    -device virtio-tablet-device \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0 \
    -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
    -device virtio-blk-device,drive=hd0 \
    -serial stdio \
    -d in_asm,int \
    -D qemu.log \
    -no-reboot
