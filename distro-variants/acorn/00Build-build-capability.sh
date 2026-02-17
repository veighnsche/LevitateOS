#!/bin/sh
set -eu

KERNEL_RELEASE_PATH="${KERNEL_RELEASE_PATH:-.artifacts/out/acorn/kernel-build/include/config/kernel.release}"
KERNEL_IMAGE_PATH="${KERNEL_IMAGE_PATH:-.artifacts/out/acorn/staging/boot/vmlinuz}"
ISO_PATH="${ISO_PATH:-.artifacts/out/acorn/s00-build/acornos-s00_build.iso}"

if [ ! -s "$KERNEL_RELEASE_PATH" ]; then
    echo "missing kernel release output: $KERNEL_RELEASE_PATH" >&2
    exit 1
fi

if [ ! -f "$KERNEL_IMAGE_PATH" ]; then
    echo "missing kernel image output: $KERNEL_IMAGE_PATH" >&2
    exit 1
fi

if [ ! -f "$ISO_PATH" ]; then
    echo "missing ISO output: $ISO_PATH" >&2
    exit 1
fi

echo "STAGE 00 PASSED"
