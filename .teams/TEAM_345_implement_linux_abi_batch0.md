# TEAM_345 — Implement Linux ABI Compatibility Batches 0-1

**Created:** 2026-01-09
**Status:** Complete

## Mission

Implement Batch 0 (Foundation) of the Linux ABI compatibility plan:
- UoW 0.1: Add `read_user_cstring()` helper
- UoW 0.2: Add `AT_FDCWD` constant

## Plan Reference

`docs/planning/linux-abi-compatibility/phase-4.md` - Batch 0

## Progress

- [x] Verify test baseline (updated golden log per Rule 4 SILVER MODE)
- [x] UoW 0.1: Add `read_user_cstring()` helper
- [x] UoW 0.2: Add `AT_FDCWD` constant
- [x] Checkpoint: Build succeeds

## Implementation Details

### UoW 0.1: read_user_cstring()

Added to `crates/kernel/src/syscall/mod.rs`:
```rust
pub fn read_user_cstring<'a>(
    ttbr0: usize,
    user_ptr: usize,
    buf: &'a mut [u8],
) -> Result<&'a str, i64>
```

- Scans for null terminator (Linux ABI)
- Returns `ENAMETOOLONG` if buffer full without null
- Returns `EFAULT` if unmapped memory
- Returns `EINVAL` if not valid UTF-8

### UoW 0.2: fcntl constants

Added `pub mod fcntl` with:
- `AT_FDCWD` (-100)
- `AT_SYMLINK_NOFOLLOW` (0x100)
- `AT_REMOVEDIR` (0x200)
- `AT_SYMLINK_FOLLOW` (0x400)
- `AT_NO_AUTOMOUNT` (0x800)
- `AT_EMPTY_PATH` (0x1000)

## Handoff

- [x] Project builds cleanly (aarch64)
- [x] All tests pass (39/39 regression)
- [x] Team file updated
- [x] Plan updated (phase-4.md, discrepancies.md)

## Batch 1: Read-Only Syscalls

### UoW 1.1: sys_openat ✅ DONE
- Changed signature to `(dirfd, pathname, flags, mode)`
- Uses `read_user_cstring()` for null-terminated paths
- Handles `AT_FDCWD` for dirfd
- Updated all ~25 userspace call sites to use new `open()` wrapper

### UoW 1.2: sys_fstat ✅ Verified
- Kernel Stat struct matches AArch64 Linux layout (128 bytes)
- Userspace uses `linux_raw_sys::general::stat` directly

### UoW 1.3: sys_getdents ✅ Verified  
- Kernel Dirent64 struct matches Linux layout (`#[repr(C, packed)]`)
- Userspace uses `linux_raw_sys::general::linux_dirent64`

### UoW 1.4: sys_getcwd ✅ Verified (with note)
- Returns length instead of pointer (documented difference)
- Works correctly for LevitateOS userspace
- Full Linux semantics deferred to avoid breaking changes

## Batch 2: Link Operations ✅ DONE

- `sys_readlinkat` → `(dirfd, pathname, buf, bufsiz)` ✅
- `sys_symlinkat` → `(target, newdirfd, linkpath)` ✅
- `sys_linkat` → `(olddirfd, oldpath, newdirfd, newpath, flags)` ✅
- `sys_utimensat` → `(dirfd, pathname, times, flags)` ✅
- `sys_unlinkat` → `(dirfd, pathname, flags)` ✅

## Batch 3: Directory Operations ✅ DONE

- `sys_mkdirat` → `(dirfd, pathname, mode)` ✅
- `sys_renameat` → `(olddirfd, oldpath, newdirfd, newpath)` ✅
- `sys_mount`/`sys_umount` ✅ Already correct (use read_user_string)

## Batch 4: Quick Fixes ✅ DONE

- `__NR_pause` ✅ Fixed architecture-specific handling
  - x86_64: Uses `linux_raw_sys::general::__NR_pause`
  - aarch64: Uses constant 236 (kernel SyscallNumber::Pause)

## Also Fixed

- `los-hal` → `los_hal` package name in simple-gpu Cargo.toml

## Final Status

- [x] All kernel syscalls updated to Linux ABI
- [x] All userspace wrappers updated with null-terminated strings
- [x] Build passes (aarch64)
- [x] All 39 regression tests pass
- [x] Plan files updated
