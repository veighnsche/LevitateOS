#!/bin/sh
set -eu

KERNEL_RELEASE_PATH="${KERNEL_RELEASE_PATH:-.artifacts/out/RalphOS/kernel-build/include/config/kernel.release}"
KERNEL_IMAGE_PATH="${KERNEL_IMAGE_PATH:-.artifacts/out/RalphOS/staging/boot/vmlinuz}"

if [ ! -s "$KERNEL_RELEASE_PATH" ]; then
    echo "missing kernel release output: $KERNEL_RELEASE_PATH" >&2
    exit 1
fi

if [ ! -f "$KERNEL_IMAGE_PATH" ]; then
    echo "missing kernel image output: $KERNEL_IMAGE_PATH" >&2
    exit 1
fi

echo "CHECKPOINT 0 PASSED"
