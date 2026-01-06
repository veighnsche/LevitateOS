#!/bin/bash
# run-term.sh - Run LevitateOS in terminal-only mode (WSL-like)
#
# Use this when you want to interact via the terminal.
# NO graphical window - pure terminal experience.
# Type directly in THIS terminal - input goes to VM.
#
# Ctrl+C sends SIGINT to VM (if supported)
# Ctrl+A X to exit QEMU

BIN="kernel64_rust.bin"
set -e

cargo xtask build all

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  LevitateOS Terminal Mode (WSL-like)                       ║"
echo "║                                                            ║"
echo "║  Type directly here - keyboard goes to VM                  ║"
echo "║  Ctrl+A X to exit QEMU                                     ║"
echo "║  Ctrl+A C to switch to QEMU monitor                        ║"
echo "╚════════════════════════════════════════════════════════════╝"

rm -f ./qmp.sock

# -nographic: No graphical display, disables SDL/GTK window
# -serial mon:stdio: Serial console + monitor multiplexed on stdio
# This gives WSL-like behavior where keyboard goes directly to serial
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -m 1G \
    -kernel "$BIN" \
    -nographic \
    -device virtio-gpu-pci,xres=1280,yres=800 \
    -device virtio-keyboard-device \
    -device virtio-tablet-device \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0 \
    -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
    -device virtio-blk-device,drive=hd0 \
    -initrd initramfs.cpio \
    -serial mon:stdio \
    -qmp unix:./qmp.sock,server,nowait \
    -no-reboot
