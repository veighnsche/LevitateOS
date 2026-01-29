# TEAM_152: Fix qcow2 Boot Failure (Empty Disk Image)

## Problem

The qcow2 image builds without errors but fails to boot with "Boot failed: not a bootable disk". Analysis shows the qcow2 file is only 196 KiB (essentially empty - just partition tables with no actual data).

## Root Cause

The qcow2 build process doesn't validate that required input files exist before starting. If `rootfs-staging` is missing or incomplete, the build completes successfully but produces an empty/minimal image (~196 KiB after compression).

Key issues:
1. **No upfront dependency checking** - kernel, initramfs, rootfs dependencies not verified before build
2. **No rootfs content validation** - check only verifies directory exists, not that it's complete
3. **No automatic verification** - `verify_qcow2` exists but isn't called by build
4. **Work directory always cleaned** - can't inspect failed builds for debugging
5. **No diagnostic output** - intermediate partition sizes not shown

## Solution

Implement comprehensive validation following the plan:

### Phase 1: Add Dependency Checking (Fail Fast)
- Check kernel exists
- Check install initramfs exists
- Validate rootfs content and minimum size

### Phase 2: Add Diagnostic Output
- Show partition image sizes after creation
- Help users spot when something goes wrong

### Phase 3: Add Automatic Verification
- Call `verify_qcow2` immediately after conversion
- Catch empty images before cleanup

### Phase 4: Preserve Work Directory on Failure
- Keep work directory if verification fails
- Allows debugging of failed builds

### Phase 5: Improve Documentation
- Update code comments with clear build process steps
- Clarify dependency order

## Files Modified

- `leviso/src/artifact/qcow2/mod.rs` - Main build function
- `leviso/src/artifact/qcow2/helpers.rs` - New helper for dir size calculation

## Status

✅ **COMPLETE** - Implementation committed in 632a87d

All phases implemented and tested:
- ✅ Phase 1: Dependency validation with fail-fast error messages
- ✅ Phase 2: Diagnostic output for partition sizes
- ✅ Phase 3: Automatic verification of qcow2 after conversion
- ✅ Phase 4: Work directory preservation for debugging
- ✅ Phase 5: Updated documentation and code comments
- ✅ All unit tests passing (14 tests)

## Implementation Details

### Added Functions
- `verify_build_dependencies()` - Validates kernel, initramfs, and rootfs exist
- `validate_rootfs_content()` - Checks critical directories and minimum 500 MB size
- `verify_qcow2_internal()` - Verifies final image isn't suspiciously small
- `calculate_dir_size()` in helpers.rs - Calculates directory size recursively

### Key Changes
1. Updated build process documentation in module comments (steps 0-9)
2. Added dependency verification before UUID generation
3. Added size diagnostics after each partition creation
4. Added automatic verification after qcow2 conversion
5. Conditional cleanup: only delete work directory on successful verification
6. Preserved work directory on failure with helpful message

### Error Messages
- Missing kernel: "Kernel not found. Run 'cargo run -- build kernel' first."
- Missing initramfs: "Install initramfs not found... (requires initramfs-installed.img)"
- Incomplete rootfs: "rootfs-staging is incomplete: [path] not found"
- Too small rootfs: "rootfs-staging seems too small (X MB)... expected at least 500 MB"
- Failed verification: Shows work directory location and commands to inspect files

## Testing

✅ Code compiles without errors
✅ All 14 unit tests pass
✅ Error handling verified with missing dependencies
✅ Diagnostic output working correctly
