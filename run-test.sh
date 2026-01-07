#!/bin/bash
# run-test.sh - Run LevitateOS internal tests
#
# TEAM_243: Test runner mode for AI agent verification
#
# Boots the OS with a test-specific initramfs that runs all internal
# tests (*_test binaries) automatically and outputs results to stdout.
#
# Usage:
#   ./run-test.sh           # Run all internal tests
#   cargo xtask run test    # Same as above

set -e
exec cargo xtask run test "$@"
