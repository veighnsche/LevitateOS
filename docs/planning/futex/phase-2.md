# Phase 2: Design — Futex Syscall

**Team:** TEAM_208  
**Feature:** Futex (Fast Userspace Mutex)  
**ROADMAP Phase:** 17a

---

## 1. Proposed Solution

### High-Level Design

Implement a minimal futex syscall with two operations:
1. **FUTEX_WAIT**: Block the calling task until the memory location's value changes
2. **FUTEX_WAKE**: Wake up to N tasks waiting on a memory location

### User-Facing Behavior

```c
// Block if *addr == expected_val. Returns 0 on wake, EAGAIN if value mismatch.
syscall(SYS_futex, addr, FUTEX_WAIT, expected_val, NULL, NULL);

// Wake up to n waiters. Returns number woken.
syscall(SYS_futex, addr, FUTEX_WAKE, n, NULL, NULL);
```

### System Behavior

```
FUTEX_WAIT:
1. Read value at user address
2. If value != expected → return EAGAIN immediately
3. Add task to wait list keyed by virtual address
4. Mark task as Blocked, yield to scheduler
5. When woken, return 0

FUTEX_WAKE:
1. Find all waiters at virtual address
2. Wake up to N of them (remove from wait list, mark Ready)
3. Return count of woken tasks
```

---

## 2. API Design

### Syscall Number

```rust
// kernel/src/syscall/mod.rs
pub enum SyscallNumber {
    // ... existing ...
    Futex = 41,  // Next available after Mount = 40
}
```

### Function Signature

```rust
// kernel/src/syscall/sync.rs
pub fn sys_futex(addr: usize, op: usize, val: usize, timeout: usize, _addr2: usize) -> i64 {
    match op {
        FUTEX_WAIT => futex_wait(addr, val as u32, timeout),
        FUTEX_WAKE => futex_wake(addr, val),
        _ => errno::EINVAL,
    }
}
```

### Constants

```rust
pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;
```

---

## 3. Data Model Changes

### Wait List Structure

```rust
// kernel/src/syscall/sync.rs
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::sync::Arc;
use los_hal::IrqSafeLock;

/// Entry representing a blocked task waiting on a futex
struct FutexWaiter {
    task: Arc<TaskControlBlock>,
}

/// Global wait list: virtual address → list of waiters
static FUTEX_WAITERS: IrqSafeLock<BTreeMap<usize, Vec<FutexWaiter>>> = 
    IrqSafeLock::new(BTreeMap::new());
```

### TaskControlBlock Changes

**None required** — we'll use the existing `TaskState::Blocked` and exclude blocked tasks from the scheduler.

---

## 4. Behavioral Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Key type: virtual or physical? | **Virtual** | No shared memory yet, simpler |
| Timeout support? | **No** (v1) | Can add later, simplifies initial impl |
| Per-process or global? | **Global** | Single address space per process currently |
| What if process exits with waiters? | **Remove from wait list** | Prevent memory leaks |
| Spurious wakeups allowed? | **Yes** | Standard futex behavior |

---

## 5. Implementation Files

### New Files

| File | Purpose |
|------|---------|
| `kernel/src/syscall/sync.rs` | Futex syscall handler |

### Modified Files

| File | Change |
|------|--------|
| `kernel/src/syscall/mod.rs` | Add `Futex = 41`, dispatch to `sync::sys_futex` |
| `kernel/src/task/scheduler.rs` | Skip `Blocked` tasks in `pick_next()` |
| `userspace/libsyscall/src/lib.rs` | Add `futex()` wrapper |

---

## 6. Implementation Steps

### Step 1: Core Futex Infrastructure
1. Create `kernel/src/syscall/sync.rs`
2. Add `FUTEX_WAITERS` global wait list
3. Implement `futex_wait()` and `futex_wake()`

### Step 2: Scheduler Integration
1. Update `pick_next()` to skip `Blocked` tasks
2. Add helper to unblock task by Arc reference

### Step 3: Syscall Wiring
1. Add `Futex = 41` to `SyscallNumber`
2. Add dispatch case in `syscall_dispatch()`
3. Add `pub mod sync;` to module

### Step 4: Userspace Wrapper
1. Add `futex()` function to `libsyscall`

---

## 7. Open Questions for User

> [!IMPORTANT]
> **Q1**: Should we add a simple userspace test program to verify futex works?
> 
> Options:
> - A) Add `userspace/futex_test/` with mutex implementation
> - B) Just verify via shell that syscall returns correctly
> - C) Defer testing to Phase 4

---

## Verification Plan

### Automated Tests

| Test | Command | Expected |
|------|---------|----------|
| Kernel builds | `cargo check -p levitate-kernel` | No errors |
| Behavior test | `cargo xtask test behavior` | Passes (no boot regression) |
| Unit test | `cargo test -p levitate-kernel` | Passes |

### Manual Verification

1. Boot kernel with `cargo xtask run default`
2. From shell, observe no crashes
3. (Future) Run futex test program if added

---

## Next Steps

→ Await user approval, then proceed to Phase 3 (Implementation)
