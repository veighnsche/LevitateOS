# Phase 3: Implementation — Levbox Completion

**Phase:** Implementation  
**Status:** In Progress  
**Team:** TEAM_209

---

## 1. Unit of Work Breakdown

### 1.1 Step 1: Kernel Infrastructure
- [ ] Add `Linkat = 42` to `SyscallNumber` enum in `@/home/vince/Projects/LevitateOS/kernel/src/syscall/mod.rs`.
- [ ] Add `vfs_link` to `@/home/vince/Projects/LevitateOS/kernel/src/fs/vfs/dispatch.rs`.
- [ ] Implement `sys_linkat` in `@/home/vince/Projects/LevitateOS/kernel/src/syscall/fs/link.rs`.

### 1.2 Step 2: Tmpfs Hard Link Support
- [ ] Implement `link` operation in `@/home/vince/Projects/LevitateOS/kernel/src/fs/tmpfs/dir_ops.rs`.
- [ ] Update `unlink` in `@/home/vince/Projects/LevitateOS/kernel/src/fs/tmpfs/dir_ops.rs` to check `nlink`.

### 1.3 Step 3: Userspace Support
- [ ] Add `SYS_LINKAT = 42` and `linkat` wrapper to `@/home/vince/Projects/LevitateOS/userspace/libsyscall/src/lib.rs`.
- [ ] Verify `ln` utility implementation in `@/home/vince/Projects/LevitateOS/userspace/levbox/src/bin/ln.rs`.

---

## 2. Unit of Work Details

### 1.1.1 UoW 1: Register Linkat Syscall
- **File:** `kernel/src/syscall/mod.rs`
- **Task:** Add `Linkat = 42` to `SyscallNumber` and update `match` statements.

### 1.1.2 UoW 2: VFS Link Dispatch
- **File:** `kernel/src/fs/vfs/dispatch.rs`
- **Task:** Implement `vfs_link(old_path, new_path)` which handles path resolution and calls `inode.ops.link`.

### 1.2.1 UoW 3: Tmpfs Link Implementation
- **File:** `kernel/src/fs/tmpfs/dir_ops.rs`
- **Task:** Implement `InodeOps::link` for Tmpfs.
