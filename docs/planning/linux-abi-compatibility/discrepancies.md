# Linux ABI Discrepancy Inventory

**TEAM_339** | 2026-01-09

## Syscall Signature Discrepancies

| Syscall | Linux Signature | LevitateOS Signature | Priority |
|---------|-----------------|----------------------|----------|
| `openat` | `(dirfd, pathname, flags, mode)` | `(path, path_len, flags)` | HIGH |
| `mkdirat` | `(dirfd, pathname, mode)` | `(dfd, path, path_len, mode)` | HIGH |
| `unlinkat` | `(dirfd, pathname, flags)` | `(dfd, path, path_len, flags)` | HIGH |
| `renameat` | `(olddirfd, oldpath, newdirfd, newpath)` | `(old_dfd, old_path, old_path_len, new_dfd, new_path, new_path_len)` | HIGH |
| `symlinkat` | `(target, newdirfd, linkpath)` | `(target, target_len, linkdirfd, linkpath, linkpath_len)` | HIGH |
| `readlinkat` | `(dirfd, pathname, buf, bufsiz)` | `(dirfd, path, path_len, buf, bufsiz)` | HIGH |
| `linkat` | `(olddirfd, oldpath, newdirfd, newpath, flags)` | `(olddfd, oldpath, oldpath_len, newdfd, newpath, newpath_len, flags)` | HIGH |
| `utimensat` | `(dirfd, pathname, times, flags)` | `(dirfd, path, path_len, times, flags)` | HIGH |
| `getcwd` | Returns pointer on success | Returns length on success | MEDIUM |
| `mount` | `(source, target, fstype, flags, data)` | `(src, src_len, target, target_len, flags)` | MEDIUM |
| `umount` | `(target, flags)` | `(target, target_len)` | MEDIUM |

## Architecture-Specific Issues

| Issue | File | Problem | Fix |
|-------|------|---------|-----|
| `__NR_pause` | `sysno.rs:58` | Hardcoded as 34 (x86_64 only) | Use arch-conditional |

## Struct Layout Verification Needed

| Struct | Kernel Location | Userspace Source | Status |
|--------|-----------------|------------------|--------|
| `Stat` | `arch/*/mod.rs` | `linux_raw_sys::general::stat` | ⚠️ Verify |
| `Termios` | `arch/*/mod.rs` | Custom | ⚠️ Verify |
| `Timespec` | `arch/*/mod.rs` | `linux_raw_sys::general::timespec` | ⚠️ Verify |
| `Dirent64` | `syscall/fs/dir.rs` | `linux_raw_sys::general::linux_dirent64` | ⚠️ Verify |

## Error Code Issues

| Issue | Location | Problem |
|-------|----------|---------|
| Duplicate errno modules | `syscall/mod.rs:14-33` | `errno` and `errno_file` overlap |
| Magic numbers | Various | `-34` instead of `ERANGE`, etc. |

## Missing Linux Constants

| Constant | Value | Needed For |
|----------|-------|------------|
| `AT_FDCWD` | -100 | dirfd parameter |
| `ENAMETOOLONG` | 36 | Path too long error |

## Implementation Order

### Batch 0: Foundation
1. Add `read_user_cstring()` helper
2. Add `AT_FDCWD`, `ENAMETOOLONG`

### Batch 1: Read-Only (Low Risk)
1. `openat` (read mode)
2. `fstat`
3. `getdents`
4. `getcwd`

### Batch 2: Link Operations
1. `readlinkat`
2. `symlinkat`
3. `linkat`
4. `utimensat`
5. `unlinkat`

### Batch 3: Directory Operations
1. `mkdirat`
2. `renameat`
3. `mount`/`umount`

### Batch 4: Quick Fixes
1. `__NR_pause` arch fix
2. errno consolidation
3. Termios verification

### Batch 5: Struct Verification
1. Stat alignment
2. Other structs
