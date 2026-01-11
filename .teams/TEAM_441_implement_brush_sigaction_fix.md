# TEAM_441: Implement Brush Sigaction Fix

## Objective

Implement the brush-requirements plan to fix `rt_sigaction` syscall format mismatch.

## Status: COMPLETE ✅

## Plan Reference

`docs/planning/brush-requirements/` (reviewed and strengthened by TEAM_440)

## UoWs Executed

| UoW | Description | Status |
|-----|-------------|--------|
| 1.1 | Add arch-specific sigaction struct definitions | ✅ Complete |
| 2.1 | Update task signal storage with SignalAction | ✅ Complete |
| 3.1 | Rewrite sys_sigaction with arch conditionals | ✅ Complete |
| 4.1 | Update dispatcher to pass 4 args | ✅ Complete |
| 5.1 | Add TODO for 64-bit mask upgrade | ✅ Complete |
| 6.1 | Run tests and verify | ✅ Complete |

## Files Modified

| File | Change |
|------|--------|
| `crates/kernel/syscall/src/signal.rs` | Added KernelSigaction structs (arch-specific), rewrote sys_sigaction with proper struct parsing |
| `crates/kernel/syscall/src/lib.rs` | Updated dispatcher to pass 4 args to sys_sigaction |
| `crates/kernel/sched/src/lib.rs` | Added SignalAction struct, updated signal_handlers to [SignalAction; 64] |
| `crates/kernel/sched/src/thread.rs` | Updated thread creation to use SignalAction |

## Verification

- ✅ All tests pass (`cargo test --workspace`)
- ✅ Kernel builds (`cargo xtask build kernel`)
- ✅ Both x86_64 and aarch64 architectures supported

## Handoff Notes

The `rt_sigaction` syscall now properly:
1. Reads `struct sigaction` from userspace pointer (arg1)
2. Parses handler, flags, restorer (x86_64 only), and mask
3. Writes old action to oldact pointer (arg2) if provided
4. Validates sigsetsize (arg3) is 8 bytes

**Next Steps:**
- Crash persists at 0x6aa71f (Rust panic/abort)
- Child thread from clone() is never scheduled before parent crashes
- Possible causes: child thread context issue, tokio async runtime expectation mismatch

## Additional Fixes Applied

### TIOCGWINSZ ioctl (0x5413)
- Added terminal window size ioctl returning 80x24 default
- File: `crates/kernel/syscall/src/fs/fd.rs`

## Remaining Issue

Brush still crashes immediately after:
1. clone() creates child thread (TID 3)
2. getuid() returns 0
3. Crash at ud2 (Rust panic)

The child thread is added to scheduler but never runs - parent crashes first. This is likely:
1. Tokio expects child thread to signal readiness
2. Parent panics when child doesn't respond
3. Or there's an issue with thread context setup on x86_64

**Investigation needed:** Why does brush panic after clone/getuid?

