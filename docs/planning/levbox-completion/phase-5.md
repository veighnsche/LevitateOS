# Phase 5: Polish and Handoff — Levbox Completion

**Phase:** Polish and Handoff  
**Status:** In Progress  
**Team:** TEAM_209

---

## 1. Summary of Changes

### 1.1 Kernel Changes
- **Refcounting**: Added `nlink: u32` to `TmpfsNode` to track hard links.
- **Syscall**: Registered `Linkat = 42` and implemented `sys_linkat`.
- **VFS Layer**: Implemented `vfs_link` in `dispatch.rs`.
- **Tmpfs Operations**: 
    - Implemented `InodeOps::link` for `TmpfsDirOps`.
    - Updated `unlink` and `rmdir` to correctly decrement and check `nlink`.
- **Architecture**: Added `arg6` to `SyscallFrame` to support 7-argument syscalls.
- **Cleanup**: Fixed `sys_futex` call signature in `syscall/mod.rs`.

### 1.2 Userspace Changes
- **libsyscall**: Added `SYS_LINKAT` constant and `linkat` wrapper.
- **levbox**: Confirmed packaging of all 10 utilities in `initramfs`.

---

## 2. Verification Results

- [x] Kernel builds for `aarch64-unknown-none`.
- [x] Userspace utilities built and included in `initramfs.cpio`.
- [x] `make_initramfs.sh` successfully adds: `cat`, `ls`, `pwd`, `mkdir`, `rmdir`, `rm`, `mv`, `cp`, `touch`, `ln`.

---

## 3. Handoff Notes
- **Future Teams**: The `utimensat` syscall still ignores `dirfd` and `flags` (except for path resolution which is relative to root/cwd).
- **Regression Testing**: All levbox utilities are now in the root of the initramfs. Testing `ln` (hard link) vs `ln -s` (symlink) should confirm `nlink` behavior in `ls -l`.
