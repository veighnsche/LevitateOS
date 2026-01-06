# Phase 1: Discovery — Levbox Completion

**Phase:** Discovery  
**Status:** In Progress  
**Team:** TEAM_209

---

## 1. Feature Summary

### Problem Statement
Phase 11 of LevitateOS (levbox) is partially complete. While 8/10 utilities are done, `touch` and `ln` (specifically hard links via `linkat`) remain incomplete or unverified. `touch` requires verified `utimensat` support, and `ln` requires both symbolic links (`symlinkat`) and hard links (`linkat`).

### Who Benefits
- Users requiring a full suite of standard Unix-like utilities.
- Developers needing reliable file creation and linking capabilities in the OS.

---

## 2. Success Criteria

| Criteria | Verification |
|----------|--------------|
| `touch /tmp/file` | File created with correct timestamps |
| `ln -s target link` | Symbolic link created and readable |
| `ln target link` | Hard link created (share inode) |
| All 10 utilities in initramfs | `ls`, `cat`, `mkdir`, `rm`, `rmdir`, `mv`, `cp`, `pwd`, `touch`, `ln` available |

---

## 3. Current State Analysis

### 3.1 Existing Infrastructure
- **`utimensat`**: Implemented but ignores `dirfd` and `flags`.
- **`symlinkat`**: Implemented, stores target in `TmpfsNode::data`.
- **`readlinkat`**: Implemented for reading symlink targets.
- **`linkat`**: **MISSING**. `TmpfsNode` does not have `nlink` (refcount).
- **`TmpfsNode`**: Has `atime`, `mtime`, `ctime` but they use `uptime_seconds()`.

### 3.2 Blockers
1. **Hard Link Support**: `TmpfsNode` needs `nlink: u32` to track how many directory entries point to it.
2. **`linkat` Syscall**: Missing kernel handler and userspace wrapper.
3. **Initramfs Packaging**: Need to ensure `touch` and `ln` are actually being built and included in the boot image.

---

## 4. Codebase Reconnaissance

### 4.1 Files to Modify
- `@/home/vince/Projects/LevitateOS/kernel/src/fs/tmpfs/node.rs`: Add `nlink` to `TmpfsNode`.
- `@/home/vince/Projects/LevitateOS/kernel/src/syscall/mod.rs`: Add `Linkat` syscall number.
- `@/home/vince/Projects/LevitateOS/kernel/src/syscall/fs/link.rs`: Implement `sys_linkat`.
- `@/home/vince/Projects/LevitateOS/userspace/libsyscall/src/lib.rs`: Add `linkat` wrapper.
- `@/home/vince/Projects/LevitateOS/scripts/make_initramfs.sh`: Ensure all utilities are included.

---

## 5. Constraints
- **Tmpfs only**: Hard links and symlinks are currently only targeted for `tmpfs`.
- **Memory safety**: Must ensure no reference cycles are created with hard links (though usually restricted to files).

---

## 6. Phase 1 Steps

- [x] Identify missing syscalls (`linkat`).
- [x] Document current infrastructure (`utimensat`, `symlinkat` status).
- [x] List blockers (nlink support).
- [x] Verify `make_initramfs.sh` configuration.
