#!/bin/bash
# run-term.sh - Run LevitateOS with serial output in this terminal
#
# TEAM_387: Direct terminal mode - no VNC/screenshots needed
#
# Usage:
#   ./run-term.sh              # Run x86_64 in terminal
#   ./run-term.sh --aarch64    # Run AArch64 in terminal
#
# Controls:
#   Ctrl+A X - Exit QEMU
#   Ctrl+A C - Switch to QEMU monitor

set -e

exec cargo xtask run --term "$@"
