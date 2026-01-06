# TEAM_198: Implement Remaining Levbox Syscalls

**Task:** Implement Phase 3 of levbox-remaining-syscalls plan  
**Plan:** `docs/planning/levbox-remaining-syscalls/phase-3.md`  
**Started:** 2026-01-06  
**Status:** ✅ Complete

## Steps

| Step | Description | Status |
|------|-------------|--------|
| 1 | Add levbox utilities to initramfs | ✅ |
| 2 | Add timestamps to TmpfsNode | ✅ |
| 3 | Implement sys_utimensat | ✅ |
| 4 | Create touch utility | ✅ |
| 5 | Add symlink support to tmpfs | ✅ |
| 6 | Implement sys_symlinkat | ✅ |
| 7 | Create ln utility | ✅ |

## Implementation Summary

### Kernel Changes

| File | Changes |
|------|---------|
| `kernel/src/fs/tmpfs.rs` | Added atime/mtime/ctime fields, Symlink node type, create_symlink(), update_timestamps() |
| `kernel/src/syscall/mod.rs` | Added Utimensat (88) and Symlinkat (36) syscall numbers, Stat struct extended with timestamps |
| `kernel/src/syscall/fs.rs` | Implemented sys_utimensat() and sys_symlinkat() |
| `kernel/src/syscall/time.rs` | Added uptime_seconds() helper |

### Userspace Changes

| File | Changes |
|------|---------|
| `userspace/libsyscall/src/lib.rs` | Added utimensat() and symlinkat() wrappers, UTIME_NOW/UTIME_OMIT constants |
| `userspace/levbox/src/bin/touch.rs` | **NEW** - touch utility with -a, -c, -m flags |
| `userspace/levbox/src/bin/ln.rs` | **NEW** - ln utility with -s, -f flags |
| `userspace/levbox/Cargo.toml` | Added touch and ln binaries |
| `scripts/make_initramfs.sh` | Updated to include all levbox utilities including touch and ln |

## Notes

- Previous team: TEAM_197 reviewed and approved this plan
- Tmpfs already implemented by TEAM_194
- Hard links (`linkat`) intentionally deferred - complexity not justified for Phase 11
- Pre-existing lint warning about `offset` field in fd_table.rs is unrelated to this work

## Handoff Checklist

- [x] All steps complete
- [x] Userspace builds cleanly
- [x] Initramfs includes touch and ln
- [x] CHECKLIST.md updated
- [x] Team file updated
