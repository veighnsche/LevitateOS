#!/bin/bash
# Download Alpine Linux ISO images for testing
# TEAM_325: Required for screenshot integration tests

set -e

ALPINE_VERSION="3.20.0"
ALPINE_MIRROR="https://dl-cdn.alpinelinux.org/alpine/v3.20/releases"

cd "$(dirname "$0")"

echo "📥 Downloading Alpine Linux v${ALPINE_VERSION} images..."

# x86_64
if [ ! -f "alpine-virt-${ALPINE_VERSION}-x86_64.iso" ]; then
    echo "  Downloading x86_64..."
    curl -L -o "alpine-virt-${ALPINE_VERSION}-x86_64.iso" \
        "${ALPINE_MIRROR}/x86_64/alpine-virt-${ALPINE_VERSION}-x86_64.iso"
else
    echo "  x86_64 already exists"
fi

# aarch64
if [ ! -f "alpine-virt-${ALPINE_VERSION}-aarch64.iso" ]; then
    echo "  Downloading aarch64..."
    curl -L -o "alpine-virt-${ALPINE_VERSION}-aarch64.iso" \
        "${ALPINE_MIRROR}/aarch64/alpine-virt-${ALPINE_VERSION}-aarch64.iso"
else
    echo "  aarch64 already exists"
fi

echo "✅ Done!"
ls -lh *.iso
