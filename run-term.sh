#!/bin/bash
# run-term.sh - Run LevitateOS with serial output in this terminal
#
# TEAM_387: Direct terminal mode - no VNC/screenshots needed
# TEAM_474: Now uses Linux kernel by default (race mode pivot).
#
# Usage:
#   ./run-term.sh              # Run x86_64 with Linux kernel in terminal
#   ./run-term.sh --aarch64    # Run AArch64 in terminal (custom kernel, Linux not ready)
#   ./run-term.sh --custom     # Run with custom LevitateOS kernel (legacy)
#
# Controls:
#   Ctrl+A X - Exit QEMU
#   Ctrl+A C - Switch to QEMU monitor

set -e

# Check for --custom flag to use legacy kernel
for arg in "$@"; do
    if [ "$arg" = "--custom" ]; then
        # Remove --custom and run without --linux
        ARGS=()
        for a in "$@"; do
            [ "$a" != "--custom" ] && ARGS+=("$a")
        done
        exec cargo xtask run --term "${ARGS[@]}"
    fi
done

# TEAM_474: Linux kernel is now the default for x86_64
exec cargo xtask run --linux --term "$@"
