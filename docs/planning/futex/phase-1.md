# Phase 1: Discovery — Futex Syscall

**Team:** TEAM_208  
**Feature:** Futex (Fast Userspace Mutex)  
**ROADMAP Phase:** 17a

---

## 1. Feature Summary

### Problem Statement
LevitateOS cannot run `std::sync::Mutex` or any userspace synchronization primitives efficiently. Currently, userspace must busy-loop (`yield` + spin) to synchronize, wasting CPU.

### Who Benefits
- **std port**: Required for `std::sync::{Mutex, RwLock, Condvar}`
- **Shell pipelines**: Enables blocking reads on pipes
- **Any multi-threaded application**

### Success Criteria
1. `sys_futex(FUTEX_WAIT)` blocks calling task until woken
2. `sys_futex(FUTEX_WAKE)` wakes N waiting tasks
3. Userspace `Mutex` implementation works without busy-waiting
4. No deadlocks or lost wakeups

---

## 2. Current State Analysis

### Without Futex
- Userspace has no way to efficiently wait for a value to change
- `sys_yield()` exists but wastes CPU cycles on polling
- No wait queues in kernel

### Existing Workarounds
- **Polling**: `while !condition { yield(); }` — inefficient
- **nanosleep**: Sleep arbitrary time — unreliable timing

---

## 3. Codebase Reconnaissance

### Modules to Touch

| File | Change |
|------|--------|
| `kernel/src/syscall/mod.rs` | Add `Futex` syscall number |
| `kernel/src/syscall/sync.rs` | New file: futex handler |
| `kernel/src/task/mod.rs` | Add blocking mechanism to TCB |
| `kernel/src/task/scheduler.rs` | Support blocked tasks |

### Existing Infrastructure

| Component | Status | Notes |
|-----------|--------|-------|
| `current_task()` | ✅ | Returns `Arc<TaskControlBlock>` |
| `yield_now()` | ✅ | Re-adds to ready queue |
| `TaskState::Blocked` | ✅ Enum exists | But not fully implemented |
| Wait list | ❌ | Need to add |
| Physical address lookup | ❌ | Need virt-to-phys for user addr |

### Tests to Consider
- `cargo xtask test behavior` — existing golden test
- New unit test for futex WAIT/WAKE cycle
- Integration test with userspace mutex

---

## 4. Constraints

| Constraint | Notes |
|------------|-------|
| No hashbrown | Use `alloc::collections::BTreeMap` instead |
| IRQ safety | Futex list needs `IrqSafeLock` |
| No address spaces yet | Simplify: use virtual address as key (single address space per process) |
| AArch64 only | No cross-arch concerns for now |

---

## 5. Reference Implementation: Redox

From `.external-kernels/redox-kernel/src/syscall/futex.rs`:

```rust
// Key insight: Physical address as HashMap key
type FutexList = HashMap<PhysicalAddress, Vec<FutexEntry>>;

// FUTEX_WAIT: Check value, block if matches
if fetched != expected { return Err(EAGAIN); }
context.block("futex");
futexes.push(FutexEntry { ... });
context::switch();

// FUTEX_WAKE: Find waiters, unblock up to N
futexes[i].context_lock.unblock();
```

### Simplifications for LevitateOS v1
1. Use **virtual address** as key (no CoW/shared memory yet)
2. Skip timeout support initially
3. Only support `FUTEX_WAIT` and `FUTEX_WAKE` (skip REQUEUE, WAIT_MULTIPLE)

---

## 6. Open Questions (for Phase 2)

1. Should blocked tasks be removed from scheduler entirely, or use a `Blocked` state?
2. How to handle process exit with pending futex waiters?
3. Do we need `FUTEX_PRIVATE_FLAG` optimization?

---

## Next Steps

→ Phase 2: Design the API, blocking mechanism, and behavioral contracts
