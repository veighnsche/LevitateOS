# Phase 1: Discovery — Graceful OOM Handling

**Team:** TEAM_388  
**Feature:** Return ENOMEM to userspace instead of kernel panic on allocation failure

---

## Feature Summary

**Problem:** When userspace programs (Eyra coreutils) request memory allocations that fail, the kernel panics instead of returning an error. This is too aggressive — userspace should be able to handle OOM gracefully.

**Example trigger:** `cat hello.txt` caused a 9MB allocation that failed, crashing the entire system.

**Who benefits:** All userspace programs that can handle allocation failures gracefully.

---

## Success Criteria

1. Userspace allocation failures return `-ENOMEM` via syscall, not kernel panic
2. Userspace programs can catch the error and handle it (print message, exit cleanly)
3. Kernel heap exhaustion (kernel's own allocations) still panics with diagnostic info
4. No regression in normal allocation behavior

---

## Current State Analysis

### How it works today

1. **Userspace requests memory** via `sys_brk` (sbrk) or `sys_mmap`
2. **Kernel allocates** from frame allocator or grows heap
3. **If allocation fails:**
   - `sys_mmap` returns `ENOMEM` ✓ (this part works)
   - `sys_sbrk` returns 0 (NULL) ✗ — needs fix to return ENOMEM
   - BUT: Rust's global allocator in userspace calls `alloc::alloc()` which panics on failure

### The real problem

The panic happens in **userspace Rust code**, not the kernel syscall path:
- Eyra coreutils use Rust's standard library
- `std::alloc` calls `sbrk` syscall to grow heap
- If `sbrk` returns ENOMEM, Rust's allocator panics with "memory allocation failed"
- This panic runs in userspace but prints via kernel serial (looks like kernel panic)

### Verification needed

- [ ] Confirm syscalls already return ENOMEM correctly
- [ ] Trace where the panic actually originates (kernel vs userspace)
- [ ] Check if Eyra has a custom allocator or uses default

---

## Codebase Reconnaissance

### Kernel syscall paths (likely already correct)

| File | Function | Notes |
|------|----------|-------|
| `syscall/mm.rs` | `sys_brk` | Returns ENOMEM on failure |
| `syscall/mm.rs` | `sys_mmap` | Returns ENOMEM on failure |
| `memory/heap.rs` | `ProcessHeap::grow` | Returns Err on bounds check |

### Kernel allocator (panics - by design)

| File | Function | Notes |
|------|----------|-------|
| `main.rs` | `alloc_error_handler` | TEAM_387 just added diagnostic OOM handler |
| `arch/*/boot.rs` | `ALLOCATOR` | linked_list_allocator for kernel heap |

### Userspace allocator (needs investigation)

| Location | Notes |
|----------|-------|
| `crates/userspace/eyra/` | Uses Eyra runtime |
| Eyra's allocator | Needs to handle sbrk ENOMEM gracefully |

---

## Constraints

1. **Kernel heap OOM must still panic** — kernel cannot recover from its own OOM
2. **Userspace OOM should not panic** — return error, let program decide
3. **Diagnostic info must be preserved** — heap state, requested size, etc.

---

## Open Questions (Phase 1)

### Q1: Where exactly does the panic originate?

The panic message showed a path in `.rustup/toolchains/` which suggests it's from Rust's standard library alloc error handler, running in userspace.

**Action:** Trace the exact panic location.

### Q2: Does Eyra have a custom allocator?

If Eyra uses a custom allocator that calls sbrk, we need to make that allocator handle ENOMEM without panicking.

**Action:** Check Eyra's allocator implementation.

---

## Phase 1 Steps

### Step 1: Verify syscall ENOMEM paths
Confirm `sys_brk` and `sys_mmap` already return ENOMEM correctly.

### Step 2: Trace the actual panic source
Determine if panic is in kernel or userspace Rust code.

### Step 3: Investigate Eyra allocator
Find how Eyra handles memory allocation and what happens on failure.
