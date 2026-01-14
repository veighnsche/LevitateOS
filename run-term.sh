#!/bin/bash
# Boot LevitateOS in QEMU with serial console only (no GUI)
# Usage: ./run-term.sh [--no-build]

set -e

if [ "$1" != "--no-build" ]; then
    cargo run --bin builder -- initramfs
fi

qemu-system-x86_64 \
    -kernel vendor/linux/arch/x86/boot/bzImage \
    -initrd build/initramfs.cpio \
    -append "console=ttyS0 rw" \
    -nographic \
    -m 512M \
    -no-reboot
