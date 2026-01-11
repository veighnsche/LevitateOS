# TEAM_442: Investigate Brush Post-Clone Crash

## Objective

Investigate why brush shell crashes at ud2 after clone() and getuid() despite sigaction fix.

## Status: PARTIAL FIX APPLIED - INVESTIGATION COMPLETE

## Bug Report (from TEAM_441)

**Symptom:**
1. clone() creates child thread (TID 3) successfully
2. Parent calls getuid() → returns 0
3. Parent crashes at ud2 (0x6aa71f) - Rust panic
4. Child thread is NEVER scheduled before crash

**Crash Location:** 0x6aa71f (file offset 0x69a71f)
```asm
69a719:       syscall        ; exit_group
69a71b:       ud2
69a71d:       ud2
69a71f:       ud2            ; <- CRASH HERE (third ud2, not first!)
```

**Key Observation:** Crash at THIRD ud2, not first - brush is JUMPING to this location (Rust panic/abort).

## Pre-Investigation Checklist

- [x] Team registered as TEAM_442
- [x] Previous team logs reviewed (TEAM_438, TEAM_440, TEAM_441)
- [x] Reproduction confirmed
- [x] Partial root cause identified

## Investigation Findings

### Issue 1: Clone Syscall Argument Order - Architecture-Specific (FIXED)

**Root Cause:** Clone syscall argument order differs between architectures, but our dispatcher was not handling this.

From Linux man page clone(2):
| Architecture | Argument Order |
|--------------|----------------|
| **x86_64** | `flags, stack, parent_tid, child_tid, tls` |
| **aarch64** | `flags, stack, parent_tid, tls, child_tid` |

Our `sys_clone` function signature matches **aarch64** order.

**Fix Applied:** `crates/kernel/syscall/src/lib.rs` - Architecture-conditional dispatch:
```rust
// TEAM_442: Architecture-specific clone argument order
#[cfg(target_arch = "x86_64")]
Some(SyscallNumber::Clone) => process::sys_clone(
    frame.arg0() as u32,
    frame.arg1() as usize,
    frame.arg2() as usize,
    frame.arg4() as usize, // tls is arg4 on x86_64
    frame.arg3() as usize, // child_tid is arg3 on x86_64
    frame,
),
#[cfg(target_arch = "aarch64")]
Some(SyscallNumber::Clone) => process::sys_clone(
    frame.arg0() as u32,
    frame.arg1() as usize,
    frame.arg2() as usize,
    frame.arg3() as usize, // tls is arg3 on aarch64
    frame.arg4() as usize, // child_tid is arg4 on aarch64
    frame,
),
```

**Reference:** Used `cargo xtask syscall fetch clone` to get authoritative Linux kernel source.

### Issue 2: CLONE_FILES Not Honored (NOT FIXED)

**Problem:** When CLONE_FILES flag is set (0x400), the child should share the parent's fd table. Our code creates a new fd table for the child instead.

**Impact:** Tokio creates epoll fd, eventfd, socketpair before clone(). Child expects to use these but has empty fd table.

**Location:** `crates/kernel/sched/src/thread.rs` line 129:
```rust
// TEAM_230: For MVP, threads get their own fd table
// TODO(TEAM_230): Share fd_table when CLONE_FILES is set
fd_table: fd_table::new_shared_fd_table(),
```

**Fix Required:** Change `SharedFdTable` from `IrqSafeLock<FdTable>` to `Arc<IrqSafeLock<FdTable>>` and share when CLONE_FILES is set. This is a larger refactor.

### Issue 3: Child Thread Never Scheduled

**Observation:** The child thread (TID 3) is added to the scheduler but never runs before the parent crashes.

**Cause:** Cooperative scheduling. Parent doesn't yield after clone(), and crashes before timer preemption.

**Note:** This may not be the root cause of the parent crash - the parent crashes in userspace (Rust panic), not due to missing child.

## Crash Analysis

The parent crashes at a Rust panic/abort sequence AFTER:
1. clone() returns 3 (child TID) - SUCCESS
2. getuid() returns 0 - SUCCESS

The crash happens in brush's userspace code, jumping to the abort sequence. Without brush's debug symbols, the exact panic reason is unknown.

**Hypothesis:** Tokio's multi-threaded runtime may be panicking due to:
1. Missing fd table sharing (child can't access epoll/eventfd)
2. Some synchronization expectation not met
3. Thread-local state corruption

## Files Modified

| File | Change |
|------|--------|
| `crates/kernel/syscall/src/lib.rs` | Fixed clone arg order for x86_64 (child_tid/tls swap) |

## Remaining Work

1. **CLONE_FILES implementation** - See `docs/planning/clone-files-fd-sharing/PLAN.md`
2. **Debug brush panic** - Need brush debug symbols or strace-like output to identify panic cause
3. **Timer preemption** - Consider if cooperative scheduling is sufficient for multi-threaded apps

## Documentation Created

- **CLONE_FILES Plan**: `docs/planning/clone-files-fd-sharing/PLAN.md`
  - Full implementation plan for fd table sharing
  - Step-by-step refactoring guide
  - Testing verification points

## Handoff Notes

- Clone argument order fix is applied and verified
- CLONE_FILES is a known TODO that will be needed for brush
- The exact cause of the parent's panic remains unknown without more debugging
- Tests pass, kernel builds successfully

## Handoff Checklist

- [x] Project builds cleanly
- [x] All tests pass
- [x] Team file updated
- [ ] Behavioral regression tests (brush still crashes, expected)
- [x] Remaining TODOs documented

