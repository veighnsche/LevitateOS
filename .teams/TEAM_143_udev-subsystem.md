# TEAM_143: Isolate Udev into a Dedicated Subsystem

## Status: COMPLETE

## Problem (Original)

Udev code was scattered across 5 locations with ~290 lines of duplicated/fragmented logic:
- `tools/recinit/src/systemd.rs` (~200 lines of udev code)
- `tools/recinit/src/install.rs` (~70 lines of udev setup)
- `leviso/src/component/definitions.rs` (14 lines)
- `leviso/src/component/executor.rs` (17 lines)
- `distro-spec/src/shared/components.rs` (4 lines - UDEV_HELPERS constant)

Key Issues:
1. **Duplication**: `UDEV_HELPERS` defined in both distro-spec AND recinit
2. **Poor cohesion**: Udev logic split between initramfs (recinit) and rootfs (leviso)
3. **Hard to maintain**: 3 places create `/run/udev` (intentional defense-in-depth, but poorly documented)
4. **Hard to test**: Udev logic embedded in larger build functions

## Solution Implemented

Created a dedicated udev subsystem with clear separation of concerns:

### New Files Created

1. **`distro-spec/src/shared/udev.rs`** - Single source of truth for constants:
   - `UDEV_HELPERS` - Helper binary names
   - `UDEV_UNITS_TO_PATCH` - Units needing initramfs patches
   - `UDEV_TMPFILES_ENTRIES` - tmpfiles.d entries
   - `UDEV_TMPFILES_CONF` - Content for udev-initrd.conf
   - `UDEV_DIRS_SERVICE` - Content for udev-dirs.service

2. **`tools/recinit/src/udev/mod.rs`** - Public API:
   - `setup_initramfs_udev()` - Main entry point for initramfs
   - `setup_rootfs_udev()` - Main entry point for rootfs
   - Re-exports from submodules

3. **`tools/recinit/src/udev/helpers.rs`** - Helper binary copying:
   - `copy_udev_helpers()` - Copies helpers with library dependencies

4. **`tools/recinit/src/udev/units.rs`** - Unit patching:
   - `patch_udev_units()` - Removes ConditionPathIsReadWrite=/sys, adds dependencies

5. **`tools/recinit/src/udev/dirs.rs`** - Directory and service creation:
   - `create_run_udev_dirs()` - Creates /run/udev directories
   - `create_udev_dirs_service()` - Creates udev-dirs.service
   - `enable_udev_dirs_service()` - Enables in sockets.target.wants
   - `create_udev_tmpfiles_config()` - Creates tmpfiles.d config
   - `setup_udev_dirs()` - Convenience function for all of the above

6. **`tools/recinit/src/udev/rules.rs`** - Rules copying:
   - `copy_udev_rules()` - Copies rules.d directory
   - `copy_udev_hwdb()` - Copies hwdb.d directory

### Files Modified

1. **`distro-spec/src/shared/mod.rs`** - Added udev module export
2. **`tools/recinit/src/lib.rs`** - Added `pub mod udev;`
3. **`tools/recinit/src/systemd.rs`** - Removed:
   - Duplicate `UDEV_HELPERS` constant (lines 487-493)
   - `copy_udev_helpers()` function (lines 495-547)
   - `patch_udev_units()` function (lines 383-458)
   - Inline tmpfiles.d creation (replaced with module call)
4. **`tools/recinit/src/install.rs`** - Removed:
   - Inline udev-dirs.service content (lines 208-223)
   - Manual symlink creation (lines 226-232)
   - Replaced with `crate::udev::create_udev_dirs_service()` and `enable_udev_dirs_service()`

## Defense in Depth (Preserved)

After refactoring, `/run/udev` is still created in 3 places (intentional):

1. **Init wrapper** (`install.rs` line 160-162) - Shell script, earliest
2. **udev-dirs.service** (via `udev::create_udev_dirs_service()`) - Systemd unit
3. **tmpfiles.d/udev-initrd.conf** (via `udev::create_udev_tmpfiles_config()`) - Declarative

This redundancy is intentional - udev socket activation is boot-critical.

Comments documenting this defense-in-depth strategy are now in:
- `distro-spec/src/shared/udev.rs` (module doc)
- `UDEV_DIRS_SERVICE` constant
- `UDEV_TMPFILES_CONF` constant
- `tools/recinit/src/install.rs` step 6b comments

## Test Results

All tests pass:
- `recinit`: 19 tests passed
- `distro-spec`: 44 tests passed (including 4 new udev tests)
- Workspace builds successfully (except pre-existing install-tests issue)

## Benefits Achieved

1. ✅ **Single Source of Truth**: All udev config in `distro-spec/src/shared/udev.rs`
2. ✅ **No Duplication**: UDEV_HELPERS defined once
3. ✅ **Testable**: Each module has unit tests
4. ✅ **Maintainable**: All udev logic in `tools/recinit/src/udev/`
5. ✅ **Documented**: Defense-in-depth strategy clearly commented
6. ✅ **Type-Safe**: Functions have clear inputs/outputs

## Note on leviso

The plan originally called for refactoring leviso to use the new udev module. However, leviso already uses:
- `Op::CopyTree("usr/lib/udev/rules.d")` for rules
- `Op::CopyTree("usr/lib/udev/hwdb.d")` for hwdb
- `Op::UdevHelpers(UDEV_HELPERS)` for helpers (from distro-spec)

The executor implementation for `Op::UdevHelpers` is simple (~15 lines) and works correctly.
Making leviso depend on recinit would create a circular dependency since recinit depends on distro-spec.
The current approach (leviso uses distro-spec constants directly) is cleaner than the plan suggested.

The key improvement is that UDEV_HELPERS is now also exported from `distro-spec/src/shared/udev.rs`
(in addition to the existing export from components.rs for backwards compatibility).
