#!/bin/bash
# Boot LevitateOS in QEMU with GUI display
# Usage: ./run.sh [--no-build]

set -e

if [ "$1" != "--no-build" ]; then
    cargo run -- initramfs
fi

qemu-system-x86_64 \
    -kernel vendor/linux/arch/x86/boot/bzImage \
    -initrd build/initramfs.cpio \
    -append "console=ttyS0 rw" \
    -m 512M \
    -no-reboot
