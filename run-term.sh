#!/bin/bash
# Boot LevitateOS in QEMU with serial console only (no GUI)
qemu-system-x86_64 \
    -kernel vendor/linux/arch/x86/boot/bzImage \
    -initrd build/initramfs.cpio \
    -append "console=ttyS0 rw" \
    -nographic \
    -m 512M \
    -no-reboot
