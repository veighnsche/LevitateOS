# TEAM_080: Switch from insmod to modprobe (Architectural Fix)

## Problem

The previous implementation used `insmod` with manual dependency ordering:
- Module dependencies duplicated in two places (Rust code AND init script)
- Manual ordering was fragile - wrong order = failed boot
- Adding new hardware support required syncing multiple files
- Exponentially worse as more modules needed (NVMe, ZFS, GPU, etc.)

## Solution

Switch to `modprobe` which automatically resolves dependencies from `modules.dep`.

## Changes

### 1. `src/initramfs/mod.rs`
- Replaced `insmod` with `modprobe` and `depmod` in SBIN_UTILS

### 2. `src/initramfs/modules.rs`
- Added MODULE_METADATA_FILES constant with all files modprobe needs:
  - modules.dep, modules.dep.bin
  - modules.alias, modules.alias.bin
  - modules.softdep, modules.symbols, etc.
- Copies all metadata files to initramfs
- Runs `depmod` during build to regenerate dependency info

### 3. `profile/init`
- Simplified module loading - just call `modprobe <module>`
- Removed manual xz decompression (modprobe handles compressed modules)
- Removed manual dependency ordering comments
- ext4 now loads mbcache/jbd2 automatically
- vfat now loads fat automatically
- sr_mod now loads cdrom automatically

## Before vs After

**Before (fragile):**
```bash
# Order matters!
load_module mbcache
load_module jbd2      # must come after mbcache
load_module ext4      # must come after jbd2
```

**After (robust):**
```bash
# modprobe handles dependencies automatically
load_module ext4  # loads mbcache, jbd2 automatically
```

## Benefits

1. **No more manual ordering** - modprobe reads modules.dep
2. **Kernel updates won't break boot** - dependency info comes from kernel
3. **Easy to add new modules** - just add to the list, no ordering needed
4. **Single source of truth** - modules.dep is authoritative

## Verification

```bash
cargo build  # ✓ Compiles
# Boot test: modules should load without errors
```
