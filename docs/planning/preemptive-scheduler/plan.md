# Timer-Based Preemptive Multitasking

## Objective

Replace cooperative multitasking with timer-based preemption. The scheduler forcibly switches tasks when their time quantum expires. This is required for a general-purpose OS - no misbehaving process should be able to starve the system.

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Userspace-only preemption** | Don't preempt kernel code - avoids lock/corruption issues |
| **Deferred reschedule pattern** | Timer sets flag, exception return path does the switch |
| **10-tick quantum (100ms)** | Conservative starting point at 100Hz timer |
| **No feature flag** | Preemption is always on - this is the target architecture |

## Current State

- Timer fires at 100Hz, handler at `levitate/src/init.rs:160-196`
- TEAM_148 disabled preemption: "IRQ handlers must NOT yield"
- PCR has reserved space at offset 40 (AArch64) / padding (x86_64)
- `check_signals()` already runs on IRQ return from userspace - same pattern

---

## Phase 1: Data Structures

Add time tracking fields to TaskControlBlock and PCR.

### 1.1 TaskControlBlock Fields

**File**: `crates/kernel/sched/src/lib.rs`

Add constant:
```rust
pub const QUANTUM_TICKS: u32 = 10;  // 100ms at 100Hz
```

Add to `TaskControlBlock` struct (after `umask` field):
```rust
/// Remaining ticks before preemption
pub ticks_remaining: AtomicU32,
/// Total CPU ticks consumed (accounting)
pub total_ticks: AtomicU64,
```

Update `Default` and `From<UserTask>` impls:
```rust
ticks_remaining: AtomicU32::new(QUANTUM_TICKS),
total_ticks: AtomicU64::new(0),
```

### 1.2 PCR Fields (AArch64)

**File**: `crates/kernel/arch/aarch64/src/cpu.rs`

Replace `_reserved` field:
```rust
/// Flag: preemption needed on return to userspace (offset 40)
pub needs_reschedule: AtomicBool,
/// Preemption disable depth counter (offset 48)
pub preempt_count: AtomicU32,
/// Padding for alignment (offset 52)
pub _reserved: [u8; 12],
```

Update `new()` to initialize both to default values.

### 1.3 PCR Fields (x86_64)

**File**: `crates/kernel/arch/x86_64/src/cpu.rs`

Replace `_padding` field:
```rust
/// Flag: preemption needed on return to userspace
pub needs_reschedule: AtomicBool,
/// Preemption disable depth counter
pub preempt_count: AtomicU32,
```

### 1.4 try_current_task Helper

**File**: `crates/kernel/sched/src/lib.rs` (after `current_task()`)

```rust
/// Get the current task if one exists, without panicking.
/// Returns None during early boot or if no task is running.
pub fn try_current_task() -> Option<Arc<TaskControlBlock>> {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    unsafe {
        let pcr = cpu::get_pcr();
        let ptr = pcr.current_task_ptr as *const TaskControlBlock;
        if !ptr.is_null() {
            let arc = Arc::from_raw(ptr);
            let cloned = arc.clone();
            let _ = Arc::into_raw(arc);
            return Some(cloned);
        }
    }
    None
}
```

### Verification

- Build succeeds for both architectures
- All existing tests pass
- No behavior change yet

---

## Phase 2: Timer Quantum Tracking

Track time consumption and set reschedule flag when quantum expires.

### 2.1 Timer Handler Update

**File**: `crates/kernel/levitate/src/init.rs` (in `TimerHandler::handle`)

After timer reload, before GPU flush:
```rust
// TEAM_XXX: Track time quantum for preemption
if let Some(task) = los_sched::try_current_task() {
    task.total_ticks.fetch_add(1, Ordering::Relaxed);

    let remaining = task.ticks_remaining.fetch_sub(1, Ordering::AcqRel);
    if remaining <= 1 {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            {
                let pcr = los_arch_aarch64::cpu::get_pcr();
                pcr.needs_reschedule.store(true, Ordering::Release);
            }
            #[cfg(target_arch = "x86_64")]
            {
                let pcr = los_arch_x86_64::cpu::get_pcr();
                pcr.needs_reschedule.store(true, Ordering::Release);
            }
        }
    }
}
```

### 2.2 Reset Quantum on Context Switch

**File**: `crates/kernel/sched/src/lib.rs` (in `switch_to`)

After `set_current_task(new_task.clone())`:
```rust
// Reset quantum for incoming task
new_task.ticks_remaining.store(QUANTUM_TICKS, Ordering::Release);
```

### Verification

- Add logging to verify quantum decrements
- Verify `needs_reschedule` flag gets set after 10 ticks
- No actual preemption yet (flag is set but not checked)

---

## Phase 3: Preemption Check (AArch64)

Check the reschedule flag on return to userspace and switch tasks if needed.

### 3.1 Add check_preemption Function

**File**: `crates/kernel/arch/aarch64/src/exceptions.rs` (after `check_signals`)

```rust
/// TEAM_XXX: Check if preemption is needed before returning to userspace.
pub fn check_preemption(_frame: &mut super::SyscallFrame) {
    // Atomically check and clear the flag
    let needs_reschedule = unsafe {
        let pcr = super::cpu::get_pcr();
        pcr.needs_reschedule.swap(false, Ordering::AcqRel)
    };

    if !needs_reschedule {
        return;
    }

    // Check preempt_count - don't preempt if kernel disabled it
    let preempt_disabled = unsafe {
        let pcr = super::cpu::get_pcr();
        pcr.preempt_count.load(Ordering::Acquire) > 0
    };
    if preempt_disabled {
        return;
    }

    let current = los_sched::current_task();
    let state = current.get_state();

    // Don't preempt blocked or exited tasks
    if state == los_sched::TaskState::Blocked || state == los_sched::TaskState::Exited {
        return;
    }

    // Yield to next task
    if let Some(next) = los_sched::scheduler::SCHEDULER.yield_and_reschedule(current) {
        los_sched::switch_to(next);
    }
}
```

### 3.2 Call from IRQ Handler

**File**: `crates/kernel/arch/aarch64/src/exceptions.rs` (in `handle_irq`)

After `check_signals(frame)`:
```rust
check_preemption(frame);
```

### Verification

- Boot kernel on AArch64
- Run two CPU-bound tasks that don't yield
- Verify both make progress (interleaved output)

---

## Phase 4: Preemption Check (x86_64)

Same implementation for x86_64 architecture.

### 4.1 Add check_preemption Function

**File**: `crates/kernel/arch/x86_64/src/lib.rs` (or exceptions module)

Same logic as AArch64, using x86_64 PCR access via `gs:[offset]`.

### 4.2 Call from IRQ Return Path

Locate the x86_64 IRQ handler that returns to userspace and add:
```rust
check_preemption(frame);
```

### Verification

- Boot kernel on x86_64
- Run two CPU-bound tasks that don't yield
- Verify both make progress

---

## Phase 5: Cleanup and Hardening

Remove legacy code and add kernel-side preemption control.

### 5.1 Remove TEAM_148 Dead Code

**File**: `crates/kernel/levitate/src/init.rs`

Remove the commented-out `yield_now()` and TEAM_148 comments:
```rust
// TEAM_070: Preemptive scheduling
// TEAM_148: Disabled preemption from IRQ context to prevent corruption.
// IRQ handlers must NOT yield. We rely on cooperative yielding in init/shell.
// crate::task::yield_now();
```

### 5.2 Add PreemptGuard API

**File**: `crates/kernel/sched/src/lib.rs`

For kernel code that must not be preempted:
```rust
/// RAII guard that disables preemption while held.
pub struct PreemptGuard {
    _private: (),
}

impl PreemptGuard {
    pub fn new() -> Self {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            {
                let pcr = los_arch_aarch64::cpu::get_pcr();
                pcr.preempt_count.fetch_add(1, Ordering::Release);
            }
            #[cfg(target_arch = "x86_64")]
            {
                let pcr = los_arch_x86_64::cpu::get_pcr();
                pcr.preempt_count.fetch_add(1, Ordering::Release);
            }
        }
        Self { _private: () }
    }
}

impl Drop for PreemptGuard {
    fn drop(&mut self) {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            {
                let pcr = los_arch_aarch64::cpu::get_pcr();
                pcr.preempt_count.fetch_sub(1, Ordering::Release);
            }
            #[cfg(target_arch = "x86_64")]
            {
                let pcr = los_arch_x86_64::cpu::get_pcr();
                pcr.preempt_count.fetch_sub(1, Ordering::Release);
            }
        }
    }
}

/// Disable preemption for the current scope.
pub fn preempt_disable() -> PreemptGuard {
    PreemptGuard::new()
}
```

### 5.3 Update Behavior Tests

Run `cargo xtask test behavior` and update golden files if output changes due to task interleaving.

### Verification

- `cargo xtask build all` succeeds
- `cargo xtask test` passes (update golden files as needed)
- Stress test: many concurrent tasks, no deadlocks or corruption
- Signal delivery still works correctly

---

## Files Modified

| File | Changes |
|------|---------|
| `crates/kernel/sched/src/lib.rs` | Add quantum fields, try_current_task(), reset on switch, PreemptGuard |
| `crates/kernel/arch/aarch64/src/cpu.rs` | Add needs_reschedule, preempt_count to PCR |
| `crates/kernel/arch/x86_64/src/cpu.rs` | Add needs_reschedule, preempt_count to PCR |
| `crates/kernel/levitate/src/init.rs` | Timer decrements quantum, sets flag, remove TEAM_148 cruft |
| `crates/kernel/arch/aarch64/src/exceptions.rs` | Add check_preemption(), call from handle_irq |
| `crates/kernel/arch/x86_64/src/lib.rs` | Add check_preemption() for x86_64 |

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Preempt while holding lock | IrqSafeLock already disables interrupts; PreemptGuard for extra safety |
| TLB inconsistency | switch_to() already handles TTBR0/CR3 switch with proper barriers |
| Signal delivery race | check_signals() runs BEFORE check_preemption() |
| FPU state corruption | cpu_switch_to() already saves/restores full FPU state |
| Early boot crash | try_current_task() returns None if no task running |

---

## Future Enhancements

After this is working:
1. **Priority levels** - Interactive (shell) vs batch (compile) scheduling
2. **Kernel preemption** - Preempt kernel code at safe points
3. **SMP support** - Per-CPU run queues, load balancing
4. **Real-time policies** - SCHED_FIFO, SCHED_RR for audio/video
