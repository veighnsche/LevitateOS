# Phase 4: Implementation and Tests

**Bug:** Brush shell crash - rt_sigaction format mismatch  
**Team:** TEAM_438  
**Parent:** `docs/planning/brush-requirements/`

## Implementation Overview

Rewrite `sys_sigaction` to properly parse Linux sigaction structs from userspace pointers.

---

## Step 1: Add sigaction Struct Definition

**File:** `crates/kernel/syscall/src/signal.rs`

**UoW 1.1:** Add arch-specific struct definitions and constants

```rust
// Signal action constants (shared)
const SIG_DFL: usize = 0;
const SIG_IGN: usize = 1;

// sigaction flags (shared)
const SA_SIGINFO: u64 = 0x00000004;
const SA_RESTART: u64 = 0x10000000;
const SA_NODEFER: u64 = 0x40000000;

#[cfg(target_arch = "x86_64")]
mod sigaction_arch {
    /// Linux sigaction struct layout for x86_64
    /// Total size: 32 bytes
    #[repr(C)]
    pub struct KernelSigaction {
        pub sa_handler: usize,      // offset 0: handler or SIG_IGN/SIG_DFL
        pub sa_flags: u64,          // offset 8: flags
        pub sa_restorer: usize,     // offset 16: signal trampoline (x86_64 only)
        pub sa_mask: u64,           // offset 24: 64-bit signal mask
    }
    
    pub const SA_RESTORER: u64 = 0x04000000;
    pub const SIGACTION_SIZE: usize = 32;
}

#[cfg(target_arch = "aarch64")]
mod sigaction_arch {
    /// Linux sigaction struct layout for aarch64
    /// Total size: 24 bytes (NO sa_restorer field!)
    #[repr(C)]
    pub struct KernelSigaction {
        pub sa_handler: usize,      // offset 0: handler or SIG_IGN/SIG_DFL
        pub sa_flags: u64,          // offset 8: flags
        pub sa_mask: u64,           // offset 16: 64-bit signal mask
    }
    
    // aarch64 does not use SA_RESTORER - kernel provides signal trampoline
    pub const SIGACTION_SIZE: usize = 24;
}

use sigaction_arch::*;
```

---

## Step 2: Update Task Signal Storage

**File:** `crates/kernel/sched/src/lib.rs`

**UoW 2.1:** Expand signal handler storage to include flags

Current:
```rust
pub signal_handlers: Mutex<[usize; 32]>,
pub signal_trampoline: AtomicUsize,
```

New:
```rust
pub signal_handlers: Mutex<[SignalAction; 64]>,
```

Where `SignalAction` is:
```rust
#[derive(Clone, Copy, Default)]
pub struct SignalAction {
    pub handler: usize,
    pub flags: u64,
    pub restorer: usize,
    pub mask: u64,
}
```

---

## Step 3: Rewrite sys_sigaction

**File:** `crates/kernel/syscall/src/signal.rs`

**UoW 3.1:** New function signature and implementation

```rust
/// TEAM_438: Proper rt_sigaction implementation with struct parsing
/// TEAM_440: Added arch-conditional handling for x86_64 vs aarch64
pub fn sys_sigaction(
    sig: i32,
    act_ptr: usize,
    oldact_ptr: usize,
    sigsetsize: usize,
) -> SyscallResult {
    // 1. Validate signal number
    if sig < 1 || sig >= 64 {
        return Err(EINVAL);
    }
    // SIGKILL and SIGSTOP cannot have handlers
    if sig == 9 || sig == 19 {
        return Err(EINVAL);
    }
    
    // 2. Validate sigsetsize (must be 8 for 64-bit sigset_t)
    if sigsetsize != 8 {
        return Err(EINVAL);
    }
    
    let task = current_task();
    let ttbr0 = task.ttbr0;
    
    // 3. If oldact_ptr is provided, write current action
    if oldact_ptr != 0 {
        let handlers = task.signal_handlers.lock();
        let old_action = &handlers[sig as usize];
        let kernel_sigaction = signal_action_to_kernel(old_action);
        write_struct_to_user(ttbr0, oldact_ptr, &kernel_sigaction)?;
    }
    
    // 4. If act_ptr is provided, read and store new action
    if act_ptr != 0 {
        let kernel_sigaction: KernelSigaction = read_struct_from_user(ttbr0, act_ptr)?;
        let new_action = kernel_to_signal_action(&kernel_sigaction);
        let mut handlers = task.signal_handlers.lock();
        handlers[sig as usize] = new_action;
        
        // x86_64 only: If SA_RESTORER is set, store the trampoline
        #[cfg(target_arch = "x86_64")]
        if new_action.flags & sigaction_arch::SA_RESTORER != 0 {
            task.signal_trampoline.store(new_action.restorer, Ordering::Release);
        }
    }
    
    Ok(0)
}

/// Convert internal SignalAction to arch-specific KernelSigaction
fn signal_action_to_kernel(action: &SignalAction) -> KernelSigaction {
    KernelSigaction {
        sa_handler: action.handler,
        sa_flags: action.flags,
        #[cfg(target_arch = "x86_64")]
        sa_restorer: action.restorer,
        sa_mask: action.mask,
    }
}

/// Convert arch-specific KernelSigaction to internal SignalAction
fn kernel_to_signal_action(k: &KernelSigaction) -> SignalAction {
    SignalAction {
        handler: k.sa_handler,
        flags: k.sa_flags,
        #[cfg(target_arch = "x86_64")]
        restorer: k.sa_restorer,
        #[cfg(target_arch = "aarch64")]
        restorer: 0, // aarch64 doesn't use sa_restorer
        mask: k.sa_mask,
    }
}
```

**UoW 3.2:** Use existing helpers for userspace struct I/O

**TEAM_440 Review:** Use existing `read_struct_from_user` and `write_struct_to_user` from `helpers.rs` instead of byte-by-byte reading. This follows existing codebase patterns.

```rust
// Use existing helper (already in crates/kernel/syscall/src/helpers.rs)
use crate::helpers::{read_struct_from_user, write_struct_to_user};

// In sys_sigaction:
if act_ptr != 0 {
    let kernel_sigaction: KernelSigaction = read_struct_from_user(ttbr0, act_ptr)?;
    // Convert to SignalAction and store...
}

if oldact_ptr != 0 {
    let old_kernel_sigaction = /* convert SignalAction to KernelSigaction */;
    write_struct_to_user(ttbr0, oldact_ptr, &old_kernel_sigaction)?;
}
```

**Note:** `KernelSigaction` must derive `Copy` and `Clone` for the helpers to work.

---

## Step 4: Update Syscall Dispatcher

**File:** `crates/kernel/syscall/src/lib.rs`

**UoW 4.1:** Fix dispatcher to pass 4 arguments

Current:
```rust
Some(SyscallNumber::SigAction) => signal::sys_sigaction(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as usize,
),
```

New:
```rust
Some(SyscallNumber::SigAction) => signal::sys_sigaction(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as usize,
    frame.arg3() as usize,  // sigsetsize
),
```

---

## Step 5: Update sigprocmask for 64-bit Masks

**File:** `crates/kernel/syscall/src/signal.rs`

**UoW 5.1:** Update blocked_signals to 64-bit

This is a lower priority change. Initial fix can keep 32-bit mask.

**TEAM_440 Review:** This is deferred but must be tracked. Add TODO:

```rust
// TODO(TEAM_438): Upgrade blocked_signals from AtomicU32 to AtomicU64
// for full 64-signal support. sys_sigprocmask also needs updating.
// See: docs/planning/brush-requirements/phase-4.md Step 5
```

---

## Step 6: Verification

**UoW 6.1:** Run tests

```bash
# Unit tests
cargo test --workspace

# Build and run
cargo xtask build kernel
cargo xtask build iso
timeout 15 cargo xtask run --term --headless 2>&1 | grep -E "sigaction|EXCEPTION"
```

**UoW 6.2:** Verify brush progress

Expected: Brush should get past signal setup and make more syscalls before any crash.

---

## Execution Order

| Step | UoW | Description | Size |
|------|-----|-------------|------|
| 1 | 1.1 | Add arch-specific struct definitions (x86_64 + aarch64) | Small |
| 2 | 2.1 | Update task signal storage with SignalAction | Medium |
| 3 | 3.1 | Rewrite sys_sigaction with arch conditionals | Medium |
| 3 | 3.2 | Use existing helpers (read/write_struct_from_user) | Small |
| 4 | 4.1 | Update dispatcher to pass 4 args | Small |
| 5 | 5.1 | Add TODO for 64-bit mask upgrade | Small |
| 6 | 6.1 | Run tests (both x86_64 and aarch64) | Small |
| 6 | 6.2 | Verify brush progress | Small |

Total: ~6-7 small-medium UoWs, can be done in 1-2 sessions.

**TEAM_440 Review:** Ensure aarch64 is tested as well as x86_64 to verify the arch-conditional code works correctly.

---

## Phase 4 Status: READY FOR IMPLEMENTATION

All UoWs defined. Ready to execute.
