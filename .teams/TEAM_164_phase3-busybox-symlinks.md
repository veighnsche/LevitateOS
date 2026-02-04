# TEAM_164: Phase 3 Task 3.3 — Busybox Symlinks

**Date**: 2026-02-04
**Iteration**: 10 (Haiku)
**Status**: ✅ COMPLETED

## What Was Done

Verified that busybox applet symlinks are correctly created during AcornOS rootfs build. This fulfills task 3.3: "Busybox symlinks created for all applets (/bin/sh → busybox, /bin/ls → busybox, etc.)".

## Key Findings

The infrastructure for busybox symlink creation was **already fully implemented**:

1. **BUSYBOX component** (src/component/definitions.rs:93-102):
   - Phase::Binaries phase
   - Two operations: bin("busybox") to copy binary, custom(CustomOp::CreateBusyboxApplets)
   - Included in ALL_COMPONENTS list

2. **Busybox custom operation** (src/component/custom/busybox.rs):
   - get_busybox_applets(): Runs `busybox --list` to enumerate applets
   - Fallback: COMMON_APPLETS hardcoded list (~100 applets) if --list fails
   - create_applet_symlinks(): Creates symlinks for all applets
   - is_sbin_applet(): Separates bin/sbin applets using SBIN_APPLETS constant (~70 applets)
   - Creates essential symlinks like /usr/bin/sh -> /usr/bin/busybox
   - Respects existing files (won't overwrite standalone binaries)

3. **Component system orchestration**:
   - BUSYBOX component executed in Phase::Binaries (phase 2)
   - After FILESYSTEM (phase 1) so directories exist
   - Before UTILITIES component which copies additional binaries

## Verification

Built rootfs with `cargo run -- build rootfs` and verified output/rootfs-staging/:

**Busybox binary**: 805k at /usr/bin/busybox ✓

**/usr/bin applet symlinks**: 114 total
- Examples: ash, sh, cat, ls, grep, awk, sed, cp, mv, rm, find, tar, gzip, etc.
- All point to /usr/bin/busybox

**/usr/sbin applet symlinks**: ~60 total
- Examples: getty, init, halt, insmod, lsmod, modinfo, modprobe, etc.
- All point to /usr/bin/busybox
- Note: /usr/sbin/init -> /bin/busybox (hardlink variation in source)

**Critical symlinks verified**:
- /usr/bin/sh -> /usr/bin/busybox ✓
- /usr/bin/ash -> /usr/bin/busybox ✓
- /usr/bin/ls -> /usr/bin/busybox ✓
- /usr/bin/cat -> /usr/bin/busybox ✓
- /usr/sbin/getty -> /usr/bin/busybox ✓

## Files Modified

None. The implementation was complete. Only verified existing functionality and updated documentation.

## Decisions & Rationale

- No code changes needed — the component system already handles busybox symlink creation
- The busybox --list mechanism is robust; fallback to hardcoded list provides safety
- Separation of bin vs sbin applets is correct and matches Alpine conventions
- Respecting existing files prevents accidental overwrites of standalone binaries

## Blockers

None. Task completed successfully.

## Next Task

Task 3.4: "OpenRC installed and configured as init system (not systemd)"
