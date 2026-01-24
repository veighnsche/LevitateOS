# TEAM_102: Cleanup Leviso and Fix Build Issues

## Objective
Clean up the `leviso` folder and fix fundamental build issues that were ignored or incorrectly handled by previous teams.

## Background
TEAM_101 tried to build an ISO and noticed a version mismatch between the custom kernel and the modules being packaged. Instead of fixing the build logic to use the correct modules, they documented the failure and "wired it up" anyway, claiming success because it might boot to a shell.

## Status: COMPLETE

- [x] Claimed Team ID 102
- [x] Initialized task and planning
- [x] Investigate changes made by TEAM_101
- [x] Identified module resolution bug in `modules.rs`
- [x] Fixed the build system to pull modules from the correct source

## Changes Made

### 1. Fixed Module Resolution in Squashfs Builder
**File:** `leviso/src/component/custom/modules.rs`

Changed the module copy function to:
- Detect custom kernel modules at `output/staging/usr/lib/modules/`
- Copy ALL modules (249) for custom kernels, not just config list
- Check `modules.builtin` to handle built-in modules (e.g., squashfs, overlay, virtio_blk)
- Fall back to Rocky modules only if custom modules don't exist

### 2. Fixed Module Resolution in Initramfs Builder
**File:** `leviso/src/artifact/initramfs.rs`

Updated `copy_boot_modules()` to:
- Check `modules.builtin` for built-in modules
- Skip module files that are built-in to the custom kernel (12 boot modules are built-in)
- Only fail if a required module is neither a file nor built-in

### 3. Fixed Hardware Compatibility Firmware Path
**Files:** `leviso/src/artifact/iso.rs`, `leviso/src/commands/build.rs`

Changed firmware verification path from:
- `output/staging/usr/lib/firmware` (incorrect - staging only has kernel)
- to `output/squashfs-root/usr/lib/firmware` (correct - squashfs has all firmware)

### 4. Cleaned Partial Build Artifacts
Removed stale cache hashes and partial kernel build from `output/`:
- `.kconfig.hash`
- `.initramfs-inputs.hash`
- `.squashfs-inputs.hash`
- `kernel-build/` (partial, no bzImage)

## Result

Successfully built ISO with:
- Kernel version: 6.18.0-levitate (custom kernel)
- 249 kernel modules matching kernel version
- 12 boot-critical modules built-in to kernel (squashfs, overlay, loop, etc.)
- All 13 hardware compatibility profiles passing
- ISO size: 544 MB

## Timeline
- **2026-01-24 10:40**: Started cleanup and planning
- **2026-01-24 12:58**: Completed all fixes, build passes all checks
