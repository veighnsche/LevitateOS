#!/bin/bash
# run-gui.sh - Run LevitateOS with QEMU window (GPU display)
#
# Use this when you want to see the graphical display.
# Keyboard input goes to the QEMU WINDOW (click on it to type).
# Serial output appears in this terminal.
#
# To exit: Close the QEMU window, or press Ctrl+A X in terminal.

BIN="kernel64_rust.bin"
set -e

cargo xtask build all

echo "Launching QEMU with GUI window..."
echo "  → Click on QEMU window to type"
echo "  → Close window or Ctrl+A X to exit"

rm -f ./qmp.sock

qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -m 1G \
    -kernel "$BIN" \
    -display sdl \
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
