#!/bin/bash
# run.sh - Canonical LevitateOS Launcher
# TEAM_326: Updated for refactored xtask commands.
# TEAM_474: Now uses Linux kernel by default (race mode pivot).
#
# Usage:
#   ./run.sh              # Run with Linux kernel (GUI mode)
#   ./run.sh --term       # Run in Terminal mode
#   ./run.sh --gdb        # Run with GDB server
#   ./run.sh --vnc        # Run with VNC display
#   ./run.sh clean        # Clean artifacts
#   ./run.sh --custom     # Run with custom LevitateOS kernel (legacy)
#
# This script delegates to the Rust build system (xtask) which handles
# compiling, image creation, and QEMU invocation correctly.

set -e

# Forward 'clean' to xtask clean
if [ "$1" = "clean" ]; then
    exec cargo xtask clean
    exit 0
fi

# Check for --custom flag to use legacy kernel
for arg in "$@"; do
    if [ "$arg" = "--custom" ]; then
        # Remove --custom and run without --linux
        ARGS=()
        for a in "$@"; do
            [ "$a" != "--custom" ] && ARGS+=("$a")
        done
        exec cargo xtask run "${ARGS[@]}"
    fi
done

# TEAM_474: Linux kernel is now the default
exec cargo xtask run --linux "$@"
