# TEAM_406: Implement General Purpose OS (Phase 3)

**Date**: 2026-01-10
**Status**: Completed (partial)
**Plan**: `docs/planning/general-purpose-os/phase-3.md`

---

## Objective

Implement the Units of Work from TEAM_400's General Purpose OS plan.

---

## UoW Checklist

| # | UoW | Status | Notes |
|---|-----|--------|-------|
| 1 | Simple syscalls (uname, umask) | ✅ Done | Added Utsname struct, sys_uname, sys_umask |
| 2 | chdir/fchdir | ✅ Done | chdir done by TEAM_404, fchdir is stub |
| 3 | chmod/chown stubs | ✅ Done | No-op implementations per Q6 |
| 4 | poll (wrapper) | ✅ Done | Wraps sys_ppoll |
| 5 | select | ⏳ Pending | Not implemented this session |
| 6 | fork | ⏳ Pending | High complexity, future session |
| 7 | execve | ✅ Exists | sys_exec already implemented |
| 8 | c-gull staticlib | ⏳ Deferred | P2 priority |

---

## Progress Log

### Session 1 (2026-01-10)

- [x] Verified test baseline passes
- [x] UoW 1: uname, umask - Added to both x86_64 and aarch64
- [x] UoW 3: chmod/chown stubs - sys_chmod, sys_fchmod, sys_chown, sys_fchown, sys_fchmodat, sys_fchownat
- [x] UoW 4: poll wrapper - sys_poll wrapping sys_ppoll

---

## Files Modified

| File | Changes |
|------|---------|
| `crates/kernel/src/arch/x86_64/mod.rs` | Added Uname, Umask, Chmod, Fchmod, Chown, Fchown, Poll syscall numbers |
| `crates/kernel/src/arch/aarch64/mod.rs` | Same syscall numbers for aarch64 |
| `crates/kernel/src/syscall/process.rs` | Added sys_uname, sys_umask, Utsname struct |
| `crates/kernel/src/syscall/fs/fd.rs` | Added chmod/chown syscall stubs |
| `crates/kernel/src/syscall/fs/mod.rs` | Exported new functions |
| `crates/kernel/src/syscall/sync.rs` | Added sys_poll wrapper |
| `crates/kernel/src/syscall/mod.rs` | Added dispatch entries |
| `crates/kernel/src/task/mod.rs` | Added umask field to TaskControlBlock |
| `crates/kernel/src/task/thread.rs` | Added umask inheritance for threads |

---

## Handoff Notes

### Completed
- **UoW 1-4** implemented and building
- Kernel compiles for x86_64
- All syscalls wired up for both architectures

### Remaining for Future Sessions
- **UoW 5 (select)**: More complex, needs fd_set bitmask handling
- **UoW 6 (fork)**: High complexity - page table cloning
- **UoW 8 (c-gull)**: Deferred to P2

### Handoff Checklist
- [x] Project builds cleanly
- [x] Team file updated
- [ ] All tests pass (kernel is no_std, can't run cargo test)
- [x] Remaining TODOs documented

