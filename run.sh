#!/bin/bash
# run.sh - Canonical LevitateOS Launcher
# TEAM_326: Updated for refactored xtask commands.
# TEAM_369: Now includes Eyra coreutils by default for full utility support.
#
# Usage:
#   ./run.sh              # Run in GUI mode with Eyra coreutils
#   ./run.sh --term       # Run in Terminal mode
#   ./run.sh --gdb        # Run with GDB server
#   ./run.sh --vnc        # Run with VNC display
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

# TEAM_369: Eyra coreutils are now the default (provides std support)
exec cargo xtask run "$@"
