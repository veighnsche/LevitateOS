# Phase 1: Understanding and Scoping

**Bug:** Brush shell crashes with INVALID OPCODE (ud2) after clone/getuid syscalls  
**Team:** TEAM_438  
**Parent:** `docs/planning/brush-requirements/`

## Bug Summary

Brush shell (a Rust-based bash-compatible shell using tokio async runtime) crashes with INVALID OPCODE exception at userspace address 0x6aa71f after successfully making ~30 syscalls including `clone()` (thread creation) and `getuid()`.

**Severity:** HIGH - Blocks brush shell from running, preventing a key userspace application.

**Impact:** 
- Brush cannot start interactive mode
- Tokio async runtime fails during initialization
- Any userspace app using tokio signal handling will likely fail

## Reproduction Status

**Reproducible:** YES - 100% reproducible

### Reproduction Steps

1. Build kernel: `cargo xtask build kernel`
2. Build ISO: `cargo xtask build iso`
3. Run: `cargo xtask run --term --headless`
4. Observe crash in logs

### Expected Behavior

Brush shell starts, displays prompt, accepts commands.

### Actual Behavior

```
[SYSCALL] clone -> 3 (creates thread TID 3)
[SYSCALL] getuid -> 0
EXCEPTION: INVALID OPCODE at 6aa71f
```

Crash at ud2 instruction (abort/panic handler) without exit_group syscall being logged.

## Context

### Code Areas Suspected

1. **`crates/kernel/syscall/src/signal.rs`** - `sys_sigaction` implementation
2. **Signal handling infrastructure** - sigaction struct format mismatch

### Recent Changes

- TEAM_438 added: socketpair, fcntl F_DUPFD_CLOEXEC, TLS setup, exception_return
- These fixes progressed brush from immediate crash to making ~30 syscalls

### Key Evidence

From brush source code analysis (`/.external-kernels/brush/`):

1. Brush uses `tokio::runtime::Builder::new_multi_thread()`
2. Tokio requires `rt_sigaction` for async signal handling (SIGCHLD listener)
3. Our `sys_sigaction` has wrong argument format

## Constraints

- **Backwards Compatibility:** Must not break existing signal handling for init process
- **Platforms:** x86_64 primary, aarch64 parity required
- **Time Sensitivity:** Medium - blocking brush shell bringup

## Open Questions (RESOLVED by TEAM_440 Review)

### Q1: Does aarch64 have the same sigaction format issue?
**ANSWER:** YES, but with a different struct layout. aarch64 sigaction is 24 bytes (no `sa_restorer` field), while x86_64 is 32 bytes. The fix must handle both architectures with `#[cfg(target_arch)]` conditionals. See Phase 3/4 for details.

### Q2: Are there other syscalls with struct pointer mismatches?
**ANSWER:** Likely yes, but out of scope for this bugfix. `sys_sigprocmask` also has potential issues with 32-bit vs 64-bit masks. TODO added for future investigation.

### Q3: What sigaction flags does tokio require (SA_RESTORER, SA_SIGINFO)?
**ANSWER:** Tokio/signal-hook-registry primarily uses:
- `SA_RESTORER` (x86_64 only) - Required for signal return trampoline
- `SA_RESTART` - Optional, restarts interrupted syscalls
- `SA_SIGINFO` - Not typically used by tokio's SIGCHLD handler

---

## Steps

### Step 1: Consolidate Bug Information ✅

Completed in this document and `BRUSH_REQUIREMENTS.md`.

### Step 2: Confirm Reproduction ✅

Confirmed reproducible. Crash occurs at same location every run.

### Step 3: Identify Suspected Code Areas ✅

Primary suspect: `crates/kernel/syscall/src/signal.rs:70-86`

```rust
// Current (WRONG):
pub fn sys_sigaction(sig: i32, handler_addr: usize, restorer_addr: usize)

// Expected (Linux format):
// arg1 = pointer to struct sigaction
// arg2 = pointer to old struct sigaction
// arg3 = sigsetsize
```

---

## Phase 1 Status: COMPLETE

Ready to proceed to Phase 2: Root Cause Analysis.
