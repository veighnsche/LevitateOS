# TEAM_125: Eliminate Sudo - Custom Initramfs Builder

**Status:** Complete
**Started:** 2026-01-26
**Goal:** Remove dracut dependency and build install initramfs without root privileges

## Background

The project has always had two initramfs approaches:
- **Live initramfs** (`build_tiny_initramfs`): Custom builder, busybox, NO root required ✓
- **Install initramfs** (`build_install_initramfs`): dracut in chroot, REQUIRES root ✗

TEAM_107 moved dracut from install-time to build-time but didn't eliminate the root requirement.

## Solution

Extend the custom builder approach (already proven for live boot) to handle installed systems.

### Key Differences: Live vs Installed Initramfs

| Aspect | Live (tiny) | Installed (full) |
|--------|-------------|------------------|
| Init | busybox shell script | systemd |
| Size | ~700KB | ~30-50MB |
| Modules | CDROM only | All storage/fs drivers |
| Firmware | None | Full firmware dir |
| Purpose | Mount ISO overlay | Boot from disk |

## Changes Made

### Phase 1: Add INSTALL_BOOT_MODULES
- Added comprehensive module list to `distro-spec/src/shared/boot_modules.rs`
- Includes filesystems (ext4, xfs, btrfs, vfat), device mapper, all storage drivers

### Phase 2: Rename squashfs-root → rootfs-staging
- Updated variable names across codebase for clarity
- Kept backward-compatible directory name in clean.rs comment

### Phase 3: Rewrite build_install_initramfs()
- Deleted `run_in_chroot()` function
- New custom builder copies modules, firmware, systemd
- NO external tools, NO root required

### Phase 4: Dracut Purge
- Deleted DRACUT component from definitions.rs
- Deleted CopyDracutModules, CreateDracutConfig from mod.rs
- Deleted dracut functions from packages.rs
- Deleted leviso/profile/etc/dracut.conf.d/levitate.conf
- Removed dracut from packages.rhai
- Removed dracut from builder.rs build order
- Cleaned up test patterns that referenced dracut

## Files Modified

1. `distro-spec/src/shared/boot_modules.rs` - Added INSTALL_BOOT_MODULES
2. `distro-spec/src/shared/mod.rs` - Exported new constant
3. `leviso/src/artifact/initramfs.rs` - Rewrote build_install_initramfs(), deleted run_in_chroot()
4. `leviso/src/artifact/rootfs.rs` - Renamed squashfs-root → rootfs-staging
5. `leviso/src/artifact/iso.rs` - Renamed references
6. `leviso/src/commands/build.rs` - Renamed references
7. `leviso/src/component/definitions.rs` - Deleted DRACUT component
8. `leviso/src/component/mod.rs` - Deleted dracut CustomOp variants
9. `leviso/src/component/custom/mod.rs` - Deleted dracut match arms
10. `leviso/src/component/custom/packages.rs` - Deleted dracut functions
11. `leviso/src/component/builder.rs` - Removed DRACUT from build order
12. `leviso/deps/packages.rhai` - Removed dracut packages
13. `testing/install-tests/src/qemu/patterns.rs` - Removed dracut error patterns
14. `testing/install-tests/src/steps/phase5_boot.rs` - Removed dracut fallback cheat
15. `testing/install-tests/docs/FAIL_FAST_POLICY.md` - Removed dracut examples
16. Deleted `leviso/profile/etc/dracut.conf.d/levitate.conf`

## Verification

```bash
# Build WITHOUT sudo (must work!)
cd /home/vince/Projects/LevitateOS
cargo run -p leviso -- build

# Check initramfs size (should be 30-50MB)
ls -lh leviso/output/initramfs-installed.img

# Verify CPIO format
file leviso/output/initramfs-installed.img
```

## Why This Approach?

| Aspect | dracut | Custom Builder |
|--------|--------|----------------|
| Root required | Yes | **No** |
| External dependency | Yes | **No** |
| Broken defaults | Yes (--kmoddir) | **N/A** |
| Control | Limited | **Full** |
| Already exists | No | **Yes** (tiny) |
