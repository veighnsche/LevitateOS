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
# TEAM_038: Use raw binary for Linux boot protocol (passes DTB in x0)
# ELF boot does NOT pass DTB - see .teams/TEAM_038_bugfix_dtb_detection.md
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a53 \
    -m 512M \
    -kernel "$BIN" \
    -display gtk \
    -device virtio-gpu-device,xres=1280,yres=800 \
    -device virtio-keyboard-device \
    -device virtio-tablet-device \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0 \
    -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
    -device virtio-blk-device,drive=hd0 \
    -initrd initramfs.cpio \
    -serial stdio \
    -d in_asm,int \
    -D qemu.log \
    -no-reboot
