#!/bin/bash
# run.sh - LevitateOS Quick Launcher
#
# This script provides the default GUI experience.
# For different modes, use:
#   ./run-gui.sh  - Opens QEMU window (click window to type)
#   ./run-term.sh - Terminal only, WSL-like (type in terminal)

echo "Starting LevitateOS in GUI mode..."
echo "  (Use ./run-term.sh for terminal-only/WSL-like mode)"
echo ""

exec bash ./run-gui.sh

