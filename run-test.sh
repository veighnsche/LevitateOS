#!/bin/bash
# run-test.sh - Run LevitateOS internal tests
#
# TEAM_243: Test runner mode for AI agent verification
# TEAM_326: Updated for refactored xtask commands.
#
# Boots the OS with a test-specific initramfs that runs all internal
# tests (*_test binaries) automatically and outputs results to stdout.
#
# Flags:
#   --aarch64  - Run on AArch64 instead of x86_64 (default)
#
# Usage:
#   ./run-test.sh             # Run all internal tests (x86_64)
#   ./run-test.sh --aarch64   # Run all internal tests (AArch64)
#   cargo xtask run --test    # Same as above

set -e

# Default to x86_64, use --aarch64 for AArch64
ARCH="x86_64"
ARGS=()
for arg in "$@"; do
    case $arg in
        --aarch64) ARCH="aarch64" ;;
        *) ARGS+=("$arg") ;;
    esac
done

exec cargo xtask run --test --arch "$ARCH" "${ARGS[@]}"
