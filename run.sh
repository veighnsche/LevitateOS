#!/bin/bash
# run.sh - Canonical LevitateOS Launcher
# TEAM_326: Updated for refactored xtask commands.
#
# Usage:
#   ./run.sh              # Run in GUI mode (Default)
#   ./run.sh --term       # Run in Terminal mode
#   ./run.sh --gdb        # Run with GDB server
#   ./run.sh --vnc        # Run with VNC display
#   ./run.sh --iso        # Force ISO boot
#   ./run.sh clean        # Clean artifacts
#
# This script delegates to the Rust build system (xtask) which handles
# compiling, image creation, and QEMU invocation correctly.

set -e

# Forward 'clean' to xtask clean
if [ "$1" = "clean" ]; then
    exec cargo xtask clean
    exit 0
fi

# TEAM_326: All run options are now flags, pass directly
exec cargo xtask run "$@"
