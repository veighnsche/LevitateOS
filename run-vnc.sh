#!/bin/bash
# run-vnc.sh - Run LevitateOS in QEMU with VNC for browser viewing
#
# TEAM_111: Created to enable browser-based QEMU display debugging
# TEAM_474: Now uses Linux kernel by default (race mode pivot).
#
# Flags:
#   --aarch64  - Run on AArch64 instead of x86_64 (custom kernel, Linux not ready)
#   --custom   - Run with custom LevitateOS kernel (legacy)
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

# Default to x86_64 with Linux
ARCH="x86_64"
USE_LINUX="--linux"
ARGS=()

for arg in "$@"; do
    case $arg in
        --aarch64) ARCH="aarch64"; USE_LINUX="" ;;  # Linux not ready for aarch64
        --custom) USE_LINUX="" ;;
        *) ARGS+=("$arg") ;;
    esac
done

NOVNC_PATH="/tmp/novnc"

echo "=== LevitateOS VNC Mode ($ARCH) ==="

# Kill any existing QEMU and websockify
pkill -f "qemu-system-aarch64" 2>/dev/null || true
pkill -f "qemu-system-x86_64" 2>/dev/null || true
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

# TEAM_474: Use xtask for consistent builds
exec cargo xtask run --vnc --arch "$ARCH" $USE_LINUX "${ARGS[@]}"
