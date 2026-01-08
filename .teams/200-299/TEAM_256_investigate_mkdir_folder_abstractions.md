# TEAM_256: Investigate mkdir/Folder Abstractions

## Status: COMPLETED ✅

## Bug Report
User reports mkdir or directory creation is failing in levbox. Unclear if tests are even running. Question: are folder abstractions correct?

## Root Cause Analysis

### Finding 1: Folder Abstractions ARE Correct ✅
The codebase has a complete, working directory abstraction:

1. **VFS Layer** (`kernel/src/fs/vfs/`):
   - `inode.rs`: `Inode::mkdir()`, `Inode::is_dir()`, mode type detection
   - `ops.rs`: `InodeOps::mkdir()`, `InodeOps::rmdir()`, `InodeOps::readdir()`, `DirEntry` struct
   - `dispatch.rs`: `vfs_mkdir()`, `vfs_rmdir()`, path resolution via dcache

2. **Tmpfs Implementation** (`kernel/src/fs/tmpfs/`):
   - `node.rs`: `TmpfsNodeType::Directory`, `TmpfsNode::new_dir()`
   - `dir_ops.rs`: Full `InodeOps` implementation including `mkdir`, `rmdir`, `readdir`, `lookup`

3. **Syscall Layer** (`kernel/src/syscall/fs/dir.rs`):
   - `sys_mkdirat()` - calls `vfs_mkdir()`
   - `sys_unlinkat()` with `AT_REMOVEDIR` flag for rmdir

4. **Userspace** (`userspace/`):
   - `libsyscall/src/fs.rs`: `mkdirat()` syscall wrapper
   - `levbox/src/bin/core/mkdir.rs`: mkdir binary

### Finding 2: Tests Weren't Running (Fixed)
The test initramfs (`xtask/src/build.rs:create_test_initramfs`) was missing:
- `suite_test_core` binary
- Core utilities: `mkdir`, `ls`, `touch`, `rm`, `cp`, `mv`, `rmdir`, `cat`

**Fix applied:** Added these binaries to `test_binaries` array.

### Finding 3: cp Utility Was Unimplemented (Fixed)
The `cp` binary had a hardcoded TODO that always returned "Read-only file system" error.

**Fix applied:** Implemented using `File::create()` and `Write::write()`.

## Fixes Applied
1. `xtask/src/build.rs`: Added `suite_test_core` and core utilities to test initramfs
2. `userspace/levbox/src/bin/core/cp.rs`: Implemented copy functionality

## Test Results After Fix
```
[TEST_RUNNER] SUMMARY
  mmap_test: PASS
  pipe_test: PASS
  signal_test: PASS
  clone_test: PASS
  interrupt_test: PASS
  tty_test: PASS
  pty_test: PASS
  suite_test_core: PASS  ← All mkdir tests pass!
  stat_test: FAIL (1)    ← Unrelated: timestamp issue
  link_test: FAIL (1)    ← Unrelated: linkat issue
  time_test: PASS
  sched_yield_test: PASS
  error_test: PASS
Total: 11/13 tests passed
```

## Remaining Issues (Not Folder-Related)
- `stat_test`: Timestamp mismatch (atime=1000 mtime=6)
- `link_test`: linkat syscall failing

## Conclusion
**The folder/directory abstractions are correctly implemented.** The perceived issue was:
1. Tests not being included in the test initramfs
2. cp utility being unimplemented (returning misleading "Read-only" error)

## Handoff Checklist
- [x] Project builds cleanly
- [x] suite_test_core tests pass (mkdir, rmdir, ls, touch, rm, cp, mv, cat)
- [x] Folder abstractions verified working
- [x] Team file updated
- [ ] stat_test and link_test failures documented as separate issues
