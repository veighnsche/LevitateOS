#!/bin/bash
# run-pixel6.sh - Run LevitateOS with Pixel 6 hardware profile
#
# TEAM_042: Pixel 6 Emulation Profile
# TEAM_474: Uses custom kernel (Linux aarch64 not ready yet).
#
# NOTE: This script is AArch64-only (Pixel 6 is an ARM device)
#
# Pixel 6 Tensor SoC:
# - 2x Cortex-X1 @ 2.80 GHz (big)
# - 2x Cortex-A76 @ 2.25 GHz (medium)
# - 4x Cortex-A55 @ 1.80 GHz (little)
# - 8GB LPDDR5
# - GICv3
#
# QEMU Mitigations:
# - cortex-a76: Exact match for medium cores, close to X1
# - 8 cores simulates big.LITTLE domains
# - 8GB RAM: Matches Pixel 6

set -e

echo "=========================================="
echo " LevitateOS - Pixel 6 Emulation Profile"
echo "=========================================="
echo "NOTE: Using custom kernel (Linux aarch64 not ready)"
echo "=========================================="

# TEAM_474: Use xtask for consistent builds
exec cargo xtask run --profile pixel6 --arch aarch64 "$@"
