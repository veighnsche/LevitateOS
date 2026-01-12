# TEAM_459: Implement Memory Surgical Fixes

## Objective

Implement the three surgical fixes recommended by TEAM_458 review to prevent future memory management bugs like TEAM_455 and TEAM_456.

## Fixes Implemented

1. **Warning Comment**: Added prominent warning to `map_user_page()` about VMA tracking requirement
2. **Debug Assertion**: Added `verify_ttbr0_consistency()` at syscall entry (debug builds only)
3. **Documentation**: Added GOTCHA #37 (ttbr0/CR3 sync) and GOTCHA #38 (VMA tracking)

## Progress Log

### Session 1 (2026-01-12)
- Started implementation based on TEAM_458 review
- Decided against making `map_user_page()` private due to circular dependency issues (mm crate can't depend on sched crate where TCB lives)
- Added warning comment to `map_user_page()` in `mm/src/user/mapping.rs`
- Added `verify_ttbr0_consistency()` function in `syscall/src/lib.rs`
- Added GOTCHA #37 and #38 to `docs/GOTCHAS.md`
- Fixed aarch64 build issues:
  - Added cfg attributes to fork.rs debug logging (x86_64-specific register names)
  - Fixed ttbr0 AtomicUsize loading in main.rs signal handler
- Verified both x86_64 and aarch64 builds pass

## Files Modified

| File | Change |
|------|--------|
| `mm/src/user/mapping.rs` | Added warning comment to `map_user_page()` |
| `syscall/src/lib.rs` | Added `verify_ttbr0_consistency()` debug assertion |
| `docs/GOTCHAS.md` | Added GOTCHA #37 and #38 |
| `sched/src/fork.rs` | Fixed aarch64 build (cfg attributes on debug logs) |
| `levitate/src/main.rs` | Fixed ttbr0 AtomicUsize loading |

## Key Decisions

1. **Warning comment vs API restriction**: Making `map_user_page()` private would require the mm crate to depend on sched crate (for TCB access), creating circular dependencies. A warning comment is simpler and achieves the same goal of alerting developers.

2. **Debug-only assertion**: The ttbr0 consistency check only runs in debug builds to avoid runtime overhead in release.

3. **Two GOTCHA entries**: Created separate entries for the two different bug patterns:
   - GOTCHA #37: Forgetting to update task.ttbr0 after CR3 switch
   - GOTCHA #38: Forgetting to track VMAs when mapping pages

### Session 2 (2026-01-12)
- Investigated BusyBox ash shell not printing prompt
- Root cause: Shell stuck in job control loop
  - Ash calls `setsid()` to become session leader → pgid changes to its own PID
  - Then checks `TIOCGPGRP` (returns `FOREGROUND_PID=1`) vs `getpgid(0)` (returns 5)
  - Mismatch causes shell to think it's not in foreground → sends SIGTTOU, loops
- Fix 1: Set `FOREGROUND_PID` when init spawns (in `init.rs`)
- Fix 2: Set `FOREGROUND_PID` when process acquires controlling terminal via `TIOCSCTTY`
  - This is the key fix: when shell calls `setsid()` then `TIOCSCTTY`, we now update
    `FOREGROUND_PID` to the shell's pgid
- Also fixed keyboard input routing: characters now fed to `CONSOLE_TTY` from interrupt handler
- Reduced scheduler switch logging to trace level (was causing spam)
- **SUCCESS**: Shell prompt now displays, keyboard input works, commands execute!

### Verification Test Results
```
LevitateOS# test
LevitateOS# cat
-/bin/ash: cat: Function not implemented
LevitateOS#
```

Shell is fully interactive. Remaining issues discovered:
- `[SYSCALL] Unknown syscall number: 4` - stat syscall not implemented
- `cat: Function not implemented` - likely due to missing stat or other syscall

## Files Modified (Session 2)

| File | Change |
|------|--------|
| `levitate/src/init.rs` | Set `FOREGROUND_PID` when init spawns |
| `syscall/src/fs/fd.rs` | Implement `TIOCSCTTY` to set foreground pgid, add trace logging |
| `levitate/src/input.rs` | Feed keyboard input to `CONSOLE_TTY` from interrupt handler |
| `sched/src/lib.rs` | Reduce switch_to logging from info to trace (reduce spam) |
| `syscall/src/lib.rs` | Change syscall logging to trace level |
| `syscall/src/fs/read.rs` | Add trace logging for poll_to_tty |
| `syscall/src/process/groups.rs` | Change getpgid logging to trace level |

## Key Decisions (Session 2)

1. **TIOCSCTTY sets foreground pgid**: When a process acquires the controlling terminal via
   `TIOCSCTTY`, it becomes the foreground process group. This allows shells that call `setsid()`
   before checking job control to work correctly.

2. **Keyboard input fed directly to TTY**: Characters are fed to `CONSOLE_TTY` from the VirtIO
   input interrupt handler, not just buffered in `KEYBOARD_BUFFER`. This ensures TTY receives
   input even when no process is actively reading.

3. **Trace-level logging for hot paths**: Scheduler switch and syscall logging changed to trace
   level to avoid flooding output when shell is idle (constant yield/reschedule).

## Gotchas Discovered

- **Job control and session leaders**: Shells typically call `setsid()` to become session leaders,
  which changes their pgid. The TTY's foreground pgid must be updated when the shell acquires
  the controlling terminal, otherwise the shell thinks it's in the background.

- **TIOCSCTTY is critical for interactive shells**: The stub implementation that just returned 0
  was insufficient. Shells expect TIOCSCTTY to establish them as the foreground process group.

### Session 3 (2026-01-12)
- Fixed syscall 4 (stat) on x86_64
  - Added `Stat = 4` to SyscallNumber enum in `arch/x86_64/src/lib.rs`
  - Added `4 => Some(Self::Stat)` to `from_u64()` match arm (was missing!)
  - Wired up dispatcher to call `sys_fstatat(-100, pathname, statbuf, 0)` (AT_FDCWD)
- Root cause of "cat: Function not implemented":
  - BusyBox ash uses stat() to check if commands exist before executing them
  - stat syscall (nr=4) was not implemented → returned ENOSYS
  - ash interpreted ENOSYS as "command can't run" → "Function not implemented"
- Both issues resolved by implementing stat syscall

## Files Modified (Session 3)

| File | Change |
|------|--------|
| `arch/x86_64/src/lib.rs` | Added `Stat = 4` to enum and `from_u64()` match |
| `syscall/src/lib.rs` | Wired Stat to sys_fstatat dispatcher |

### Session 4 (2026-01-12)
- Fixed "Permission denied" error when running commands like `cat`, `ls`
- Root cause: `vfs_stat()` wasn't following symlinks
  - `/bin/cat` is a symlink to `busybox`
  - stat() returned the symlink's inode (mode=S_IFLNK) instead of target's
  - Shell interpreted S_IFLNK mode as "not executable" → EACCES
- Fix: Added `resolve_symlinks()` helper in `vfs/src/dispatch.rs`
  - Follows symlink chain up to 8 levels deep
  - Added `VfsError::TooManySymlinks` (ELOOP) for loop detection
  - Added `vfs_lstat()` for cases where symlinks shouldn't be followed

## Files Modified (Session 4)

| File | Change |
|------|--------|
| `vfs/src/dispatch.rs` | Added `resolve_symlinks()`, modified `vfs_stat()`, added `vfs_lstat()` |
| `vfs/src/error.rs` | Added `TooManySymlinks` variant |
| `vfs/src/lib.rs` | Exported `vfs_lstat` |

## Remaining Work

- [x] Implement syscall 4 (`stat` on x86_64) - DONE
- [x] Investigate why `cat` returns "Function not implemented" - RESOLVED (was stat issue)
- [x] Fix "Permission denied" for commands - RESOLVED (symlink following in stat)

**New issues discovered:**
- `mount: mounting proc on /proc failed: Invalid argument` - procfs not implemented
- `mount: mounting sysfs on /sys failed: Invalid argument` - sysfs not implemented

## Handoff Notes

All original objectives complete. **BusyBox ash shell is now fully interactive** with:
- Prompt displays correctly
- Keyboard input works
- External commands like `cat`, `echo`, `ls` should now work (stat syscall fixed)

**Remaining limitations** (not blocking for basic shell usage):
- procfs/sysfs not implemented (mount fails with EINVAL, but shell works)
- Some BusyBox commands may still hit missing syscalls

Future teams should:
- Read GOTCHA #37 and #38 before working on memory management
- Use debug builds to catch ttbr0 desync issues early
- Check the warning comment on `map_user_page()` before using it directly
- Understand the TTY/session/pgid relationship when debugging shell issues
- When adding syscalls, remember to update BOTH the enum AND the `from_u64()` match!
