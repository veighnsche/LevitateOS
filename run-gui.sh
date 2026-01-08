#!/bin/bash
# run-gui.sh - Run LevitateOS with QEMU window (GPU display)
#
# Use this when you want to see the graphical display.
# Keyboard input goes to the QEMU WINDOW (click on it to type).
# Serial output appears in this terminal.
#
# Flags:
#   --aarch64  - Run on AArch64 instead of x86_64 (default)
#
# To exit: Close the QEMU window, or press Ctrl+A X in terminal.

set -e

# Default to x86_64, use --aarch64 for AArch64
ARCH="x86_64"
for arg in "$@"; do
    case $arg in
        --aarch64) ARCH="aarch64" ;;
    esac
done

cargo xtask build all --arch "$ARCH"

echo "Launching QEMU with GUI window ($ARCH)..."
echo "  → Click on QEMU window to type"
echo "  → Close window or Ctrl+A X to exit"

rm -f ./qmp.sock

if [ "$ARCH" = "aarch64" ]; then
    BIN="kernel64_rust.bin"
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
else
    # TEAM_292: x86_64 uses Limine ISO boot (builds ELF, not raw binary)
    ISO="levitate.iso"
    if [ ! -f "$ISO" ]; then
        echo "Building Limine ISO..."
        cargo xtask build iso --arch x86_64
    fi
    qemu-system-x86_64 \
        -M q35 \
        -cpu qemu64 \
        -m 512M \
        -boot d \
        -cdrom "$ISO" \
        -display sdl \
        -device virtio-gpu-pci,xres=1280,yres=800 \
        -device virtio-keyboard-pci \
        -device virtio-tablet-pci \
        -device virtio-net-pci,netdev=net0 \
        -netdev user,id=net0 \
        -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
        -device virtio-blk-pci,drive=hd0 \
        -serial mon:stdio \
        -qmp unix:./qmp.sock,server,nowait \
        -no-reboot
fi
