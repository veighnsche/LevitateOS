# TEAM_159: IuppiterOS Package Tiers

**Date**: 2026-02-04 (Iteration 8)
**Status**: COMPLETE
**Task**: Phase 2, task 2.6 — IuppiterOS downloads use iuppiter package tiers from distro-spec::iuppiter::packages

## Summary

Modified IuppiterOS/deps/packages.rhai to use IuppiterOS-specific package tiers instead of copying AcornOS's desktop-oriented packages.

## What Was Done

1. **Replaced AcornOS package list** with IuppiterOS appliance-focused tiers:
   - **Tier 1 (Server Core)**: eudev, firmware (intel-ucode, amd-ucode), networking (dhcpcd, iproute2, iputils, NO iwd), SSH (openssh), time sync (chrony), text processing (grep, sed, gawk)
   - **Tier 2 (Refurbishment)**: smartmontools, hdparm, sg3_utils, sdparm, nvme-cli, lsscsi, hardware enumeration (pciutils, usbutils, dmidecode), monitoring (htop, iotop), utilities (less, vim)
   - **Tier 3 (Live ISO)**: parted, xfsprogs (for installation/partitioning)

2. **Explicitly excluded** (vs AcornOS):
   - iwd, wireless-regdb (no WiFi on headless appliance)
   - sof-firmware (no audio)
   - cryptsetup (no LUKS — appliance rootfs is EROFS, data partition is plain ext4)
   - lvm2 (not needed for appliance)
   - btrfs-progs (not needed)
   - device-mapper, sgdisk, squashfs-tools (desktop/installer-only)

3. **Updated build() function** to install IuppiterOS tiers in correct order
4. **Updated is_installed() and install()** to verify IuppiterOS-specific key binaries:
   - smartctl (smartmontools)
   - hdparm
   - sg_inq (sg3_utils)
   - parted
   - curl

5. **Updated version to 1.1.0** for cache invalidation (APK will rebuild when version changes)

## Verification

Created test script to verify:
- All excluded packages are NOT present: ✅ (iwd, wireless-regdb, sof-firmware, cryptsetup, lvm2, btrfs-progs, device-mapper)
- All required packages ARE present: ✅ (smartmontools, hdparm, sg3_utils, sdparm, nvme-cli, openssh, dhcpcd)

## Files Modified

- `IuppiterOS/deps/packages.rhai`: Replaced package tiers and build/install logic

## Key Decisions

1. **Separate packages.rhai files**: Instead of making recipes parameterized, each distro has its own recipe with hardcoded package lists. This keeps the recipe language simple and makes it clear what each distro installs.

2. **Tier structure mirrors distro-spec**: The Rhai tiers directly correspond to BOOTABLE_PACKAGES, SERVER_CORE_PACKAGES, REFURBISHMENT_PACKAGES, LIVE_ISO_PACKAGES in distro-spec/src/iuppiter/packages.rs

3. **Version bump to 1.1.0**: Ensures APK packages are re-downloaded and reinstalled if the recipe changes (cache invalidation).

## No Blockers

- Cargo check passes
- No tests to run (recipe tests don't verify actual packages)
- Ready for next task

## Next Task

Task 2.7: Verify NO desktop packages in download list (final verification step for Phase 2)
