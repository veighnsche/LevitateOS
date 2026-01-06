# TEAM_174: Investigate ulib/src Bugs

## Status: Complete

## Investigation

### Bug 1: Critical - Allocator returns NULL on first allocation (FIXED)

**File:** `userspace/ulib/src/alloc.rs`
**Lines:** 79-100

**Symptom:** First heap allocation will return a null pointer (0x0).

**Root Cause Analysis:**

In `alloc()`:
1. Initial state: `head = 0`, `end = 0`
2. `aligned = (0 + align - 1) & !(align - 1)` = 0 (for any alignment)
3. `new_head = aligned + size` = size
4. Since `new_head > *end` (size > 0), we call `grow(needed)`
5. Inside `grow()`: `*head = old_break`, `*end = old_break + grow_size`
6. **BUG:** Back in `alloc()`, we use the stale `aligned` value (0) as the return pointer!
7. We return `0 as *mut u8` which is NULL

**Fix Applied:** After `grow()`, re-compute `aligned` and `new_head` with the updated `head` value.

### Issue 2: Minor - Unused field in Args struct (FIXED)

**File:** `userspace/ulib/src/env.rs`  
**Line:** 115

Removed unused `index` field from `Args` struct.

## Pre-existing Issues (Not Fixed)

- **env.rs warnings:** `static mut` references will need refactoring to use safer patterns (e.g., `OnceLock` or `Mutex`). These are warnings, not errors.
- **fs.rs:** `Read::read()` is a stub returning `NotImplemented` - documented as TODO.
- **Kernel x86_64 arch:** Incomplete module - unrelated to ulib.
- **Test harness:** no_std crates can't use standard test framework.

## Checklist
- [x] Team registered
- [x] Root cause identified
- [x] Fix implemented
- [x] Userspace builds cleanly
- [x] Handoff complete
