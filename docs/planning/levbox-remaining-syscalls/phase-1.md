# Phase 1: Discovery — Remaining Levbox Syscalls

**Phase:** Discovery  
**Status:** Complete  
**Team:** TEAM_196

---

## 1. Feature Summary

### Problem Statement

Three syscalls are missing, blocking `touch` and `ln` utilities:
- `utimensat` (88) - Set file timestamps
- `linkat` (37) - Create hard links  
- `symlinkat` (36) - Create symbolic links

Additionally, levbox utilities need to be added to the initramfs.

### Who Benefits

- Users who need `touch` to create files
- Users who need `ln` to create links
- Testing requires utilities in initramfs

---

## 2. Success Criteria

| Criteria | Verification |
|----------|--------------|
| `touch /tmp/file` | Creates file with current time |
| `touch -c /tmp/x` | Doesn't create if not exists |
| `ln -s target link` | Creates symbolic link |
| Utilities in initramfs | `ls`, `mkdir`, `rm`, `mv` available |

---

## 3. Current State Analysis

### 3.1 Existing Infrastructure

| Component | Status | Notes |
|-----------|--------|-------|
| Tmpfs | 🟢 Complete | Writable filesystem at `/tmp` |
| Time syscalls | 🟢 Complete | `clock_gettime`, `nanosleep` exist |
| Symlink node type | 🔴 Missing | TmpfsNodeType only has File/Directory |
| Hard link support | 🔴 Missing | No refcount on nodes |

### 3.2 Syscall Wrappers in libsyscall

| Syscall | Wrapper | Notes |
|---------|---------|-------|
| `utimensat` | 🔴 Missing | Need to add |
| `linkat` | 🔴 Missing | Need to add |
| `symlinkat` | 🔴 Missing | Need to add |

---

## 4. Codebase Reconnaissance

### 4.1 Files to Modify

| File | Changes Needed |
|------|----------------|
| `kernel/src/syscall/mod.rs` | Add syscall numbers |
| `kernel/src/syscall/fs.rs` | Implement handlers |
| `kernel/src/fs/tmpfs.rs` | Add symlink support, timestamp fields |
| `userspace/libsyscall/src/lib.rs` | Add wrappers |
| `userspace/levbox/src/bin/touch.rs` | Create utility |
| `userspace/levbox/src/bin/ln.rs` | Create utility |
| `scripts/make_initramfs.sh` | Add levbox utilities |

### 4.2 Reference: Linux ABI

```c
// utimensat(2)
int utimensat(int dirfd, const char *pathname,
              const struct timespec times[2], int flags);
// times[0] = access time, times[1] = modification time
// UTIME_NOW = set to current time
// UTIME_OMIT = don't change

// symlinkat(2)  
int symlinkat(const char *target, int newdirfd, const char *linkpath);

// linkat(2)
int linkat(int olddirfd, const char *oldpath,
           int newdirfd, const char *newpath, int flags);
```

---

## 5. Constraints

### 5.1 Simplifications for v1

- **Hard links**: Defer to future. Requires refcount on tmpfs nodes.
- **Symlink resolution**: Simple - store target path, don't resolve on open.
- **Timestamps**: Store in TmpfsNode, update on create/modify.

### 5.2 Dependencies

- Tmpfs must be initialized (✅ done)
- Time syscalls must work (✅ done)

---

## 6. Phase 1 Steps

- [x] Identify missing syscalls
- [x] Document current infrastructure
- [x] List files to modify
- [x] Define simplifications

---

## Next Phase

Proceed to **Phase 2: Design** to define:
- Syscall signatures
- TmpfsNode extensions for symlinks/timestamps
- Error handling
