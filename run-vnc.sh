#!/bin/bash
# run-vnc.sh - Run LevitateOS in QEMU with VNC for browser viewing
#
# TEAM_111: Created to enable browser-based QEMU display debugging
#
# Usage:
#   1. Run this script: ./run-vnc.sh
#   2. Open browser to http://localhost:6080/vnc.html
#   3. Click Connect to see QEMU display
#
# Requirements:
#   pip3 install websockify
#   git clone https://github.com/novnc/noVNC /tmp/novnc

set -e

NOVNC_PATH="/tmp/novnc"

echo "=== LevitateOS VNC Mode ==="
echo "Building kernel..."
cargo build -p levitate-kernel --release --target aarch64-unknown-none --features verbose

ELF="target/aarch64-unknown-none/release/levitate-kernel"
BIN="kernel64_rust.bin"

echo "Converting to raw binary..."
aarch64-linux-gnu-objcopy -O binary "$ELF" "$BIN"

# Kill any existing QEMU and websockify
pkill -f "qemu-system-aarch64" 2>/dev/null || true
pkill -f "websockify" 2>/dev/null || true
sleep 0.5

# Check for noVNC
if [ ! -d "$NOVNC_PATH" ]; then
    echo "Downloading noVNC..."
    git clone --depth 1 https://github.com/novnc/noVNC.git "$NOVNC_PATH"
fi

# Start websockify to serve noVNC and proxy to VNC
echo "Starting websockify + noVNC..."
~/.local/bin/websockify --web="$NOVNC_PATH" 6080 localhost:5900 &
WEBSOCKIFY_PID=$!
sleep 1

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  🌐 Open browser to: http://localhost:6080/vnc.html            ║"
echo "║  📺 Click 'Connect' to see QEMU display                        ║"
echo "║                                                                 ║"
echo "║  Serial console is in THIS terminal (type here for input)      ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Cleanup QMP socket
rm -f ./qmp.sock

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    kill $WEBSOCKIFY_PID 2>/dev/null || true
}
trap cleanup EXIT

echo "Launching QEMU with VNC on :5900..."
# Ensure disk image exists
# TEAM_121: Use xtask to ensure disk image is correctly partitioned and populated
cargo xtask build

qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -m 1G \
    -kernel "$BIN" \
    -display none \
    -vnc :0 \
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
