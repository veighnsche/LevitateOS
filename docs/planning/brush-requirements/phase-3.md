# Phase 3: Fix Design and Validation Plan

**Bug:** Brush shell crash - rt_sigaction format mismatch  
**Team:** TEAM_438  
**Parent:** `docs/planning/brush-requirements/`

## Root Cause Summary

**What's Wrong:** `sys_sigaction` interprets `struct sigaction` pointers as direct handler addresses.

**Where:** `crates/kernel/syscall/src/signal.rs:70-86`

**Current (broken):**
```rust
pub fn sys_sigaction(sig: i32, handler_addr: usize, restorer_addr: usize) -> SyscallResult {
    // Treats handler_addr as the handler itself
    handlers[sig as usize] = handler_addr;  // WRONG: this is a pointer to struct!
}
```

**Required (Linux format):**
```c
int rt_sigaction(int signum, 
                 const struct sigaction *act,    // pointer to new action
                 struct sigaction *oldact,       // pointer to store old action
                 size_t sigsetsize);             // size of sigset_t (typically 8)

struct sigaction {
    void (*sa_handler)(int);      // or sa_sigaction if SA_SIGINFO
    unsigned long sa_flags;       // SA_SIGINFO, SA_RESTORER, SA_RESTART, etc.
    void (*sa_restorer)(void);    // signal return trampoline
    sigset_t sa_mask;             // 64-bit mask of signals to block
};
```

## Fix Strategy

### Approach: Rewrite sys_sigaction with Proper Struct Parsing

1. Read `struct sigaction` from userspace pointer
2. Parse and store all fields properly
3. Write old sigaction to `oldact` if provided
4. Handle `SA_RESTORER` flag for signal trampolines

### Key Changes

| Component | Current | Fixed |
|-----------|---------|-------|
| Function signature | `(sig, handler, restorer)` | `(sig, act_ptr, oldact_ptr, sigsetsize)` |
| Handler storage | Direct address | Parsed from struct |
| Flags storage | None | New field in task |
| Mask storage | None | New field in task (64-bit) |
| Old action return | None | Write struct to oldact_ptr |

## Reversal Strategy

**How to Revert:**
1. `git revert` the commit(s)
2. Or restore original `sys_sigaction` implementation

**Signals to Revert:**
- Init process stops working
- Existing signal tests fail
- New implementation causes kernel panics

**Clean Undo Steps:**
1. Keep backup of original `signal.rs`
2. Run full test suite before and after
3. Verify init process still works

## Test Strategy

### New Tests Required

1. **Unit test:** `sys_sigaction` with struct pointer parsing
2. **Integration test:** Signal handler registration and invocation
3. **Behavioral test:** Verify brush makes it past signal setup

### Existing Tests to Verify

1. Run `cargo test --workspace` - all must pass
2. Run golden boot test - init must still work
3. Manual test: init process signal handling

### Edge Cases

1. `act` is NULL (query current action only)
2. `oldact` is NULL (don't return old action)
3. Invalid signal numbers
4. Invalid pointers (EFAULT)
5. sigsetsize != 8 (EINVAL)

## Impact Analysis

### API Changes

- Syscall argument interpretation changes
- Old callers expecting direct handler passing will break
- This is acceptable: old format was non-standard

### Affected Modules

1. `syscall/src/signal.rs` - main changes
2. `syscall/src/lib.rs` - dispatcher args
3. `sched/src/lib.rs` - may need new task fields for sa_flags, sa_mask

### Performance

- Minimal impact: one struct read from userspace per sigaction call
- Signal handling is infrequent

---

## Steps

### Step 1: Define Fix Requirements

**Correct Behavior:**
1. Read sigaction struct from `act` pointer if non-NULL
2. Store `sa_handler`, `sa_flags`, `sa_restorer`, `sa_mask` in task
3. If `oldact` non-NULL, write previous values to that pointer
4. Return 0 on success, negative errno on failure

**Invariants:**
- Signal numbers 1-31 (standard) or 1-64 (realtime) valid
- SIGKILL (9) and SIGSTOP (19) cannot have handlers
- sa_mask must be 64-bit compatible

### Step 2: Define sigaction Struct Layout

**IMPORTANT: Layout differs between architectures!**

#### x86_64 Linux `struct sigaction` (kernel format):
```c
struct sigaction {
    __sighandler_t sa_handler;    // offset 0, 8 bytes
    unsigned long sa_flags;        // offset 8, 8 bytes
    __sigrestore_t sa_restorer;   // offset 16, 8 bytes
    sigset_t sa_mask;             // offset 24, 8 bytes
};  // Total: 32 bytes
```

#### aarch64 Linux `struct sigaction` (kernel format):
```c
struct sigaction {
    __sighandler_t sa_handler;    // offset 0, 8 bytes
    unsigned long sa_flags;        // offset 8, 8 bytes
    sigset_t sa_mask;             // offset 16, 8 bytes
};  // Total: 24 bytes (NO sa_restorer field!)
```

**Key Difference:** aarch64 does NOT have `sa_restorer` field. Signal return is handled differently (kernel provides trampoline automatically).

**Key Flags:**
- `SA_RESTORER` (0x04000000) - sa_restorer field is valid **(x86_64 ONLY)**
- `SA_SIGINFO` (0x00000004) - use sa_sigaction, not sa_handler
- `SA_RESTART` (0x10000000) - restart syscalls after handler

### Step 3: Implementation Plan

See `phase-4.md` for detailed implementation steps.

---

## Phase 3 Status: COMPLETE

Fix design documented. Ready for Phase 4: Implementation.
