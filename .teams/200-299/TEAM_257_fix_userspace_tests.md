# Team 257 - Fix userspace tests (stat_test & link_test)

## Status
- [ ] Investigate `stat_test` failure (Timestamp mismatch)
- [ ] Investigate `link_test` failure (`linkat` syscall issue)
- [ ] Implement fix for `stat_test`
- [ ] Implement fix for `link_test`
- [ ] Verify fixes

## Progress
- Initializing team 257.
- Fixed `sys_utimensat` offset bug in `@/home/vince/Projects/LevitateOS/kernel/src/syscall/fs/link.rs` (was using 16 instead of 8 for u64 offsets).
- Refactored `TmpfsNode` to remove internal name and use `TmpfsDirEntry` for directory children in `@/home/vince/Projects/LevitateOS/kernel/src/fs/tmpfs/node.rs`.
- Updated `TmpfsDirOps` in `@/home/vince/Projects/LevitateOS/kernel/src/fs/tmpfs/dir_ops.rs` to handle names in directory entries.
- Fixed `sys_readlinkat` to ignore `dirfd` and use `copy_user_string`.
- Cleaned up path resolution in `sys_mkdirat`, `sys_unlinkat`, `sys_renameat`, and `sys_linkat`.
- Verified all 13 tests passed, including `stat_test` and `link_test`.

## Handoff Checklist
- [x] Project builds cleanly
- [x] All tests pass (13/13 in internal test suite)
- [x] Behavioral regression tests pass
- [x] Team file updated
- [x] Remaining TODOs documented (None)
