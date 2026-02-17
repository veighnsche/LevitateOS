#!/bin/sh
set -eu

# 00Build evidence: verify kernel + ISO capability only.
KERNEL_RELEASE_PATH="${KERNEL_RELEASE_PATH:-.artifacts/out/levitate/kernel-build/include/config/kernel.release}"
KERNEL_IMAGE_PATH="${KERNEL_IMAGE_PATH:-.artifacts/out/levitate/staging/boot/vmlinuz}"
ISO_PATH="${ISO_PATH:-.artifacts/out/levitate/levitateos-x86_64-s00_build.iso}"

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
