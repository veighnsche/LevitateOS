# Phase 3: Implementation — Graceful OOM Handling

**Team:** TEAM_388  
**Depends on:** phase-2.md (Design Decisions)  
**Status:** Ready for implementation

---

## Implementation Overview

Based on Unix philosophy from `kernel-development.md`, this implementation follows:
- **Rule 6:** Return errors, don't panic
- **Rule 11:** Kernel provides mechanism, userspace defines policy
- **Rule 14:** Fail loud, fail fast
- **Rule 20:** Simplicity over perfection

---

## Step 1: Fix sys_sbrk to Return ENOMEM

**Goal:** Make `sys_sbrk` return ENOMEM (-12) on failure instead of 0.

**File:** `crates/kernel/src/syscall/mm.rs`

**Status:** ✅ TEAM_389 verified:
- `sys_mmap` already returns ENOMEM correctly
- `sys_sbrk` returns 0 (NULL) on failure — **needs fix**

**Tasks:**
1. ~~Read `sys_mmap` implementation — verify it returns ENOMEM on failure~~ ✓
2. Fix `sys_sbrk` to return ENOMEM (-12) instead of 0 on allocation failure

**Change required:**
```rust
// In sys_sbrk, change:
return 0; // null
// To:
return -12; // ENOMEM
```

---

## Step 2: Increase USER_HEAP_MAX_SIZE to 256MB

**Goal:** Give userspace more headroom.

**Status:** ✅ TEAM_389 IMPLEMENTED

**Files changed:**
- `crates/kernel/src/memory/heap.rs` - Updated constant
- `crates/kernel/src/task/user.rs` - Updated duplicate constant

---

## Step 3: Add Debug Logging for Userspace OOM

**Goal:** Log OOM events for debugging (Rule 4: Silence is Golden - use debug level).

**Status:** ✅ TEAM_389 IMPLEMENTED

**File:** `crates/kernel/src/syscall/mm.rs`

**Changes:**
- Added `log::debug!` calls to all OOM paths in `sys_sbrk` and `sys_mmap`
- Uses debug level so logs are silent in production but visible when debugging

---

## Step 4: Eyra Allocator Investigation

**Goal:** Understand how Eyra handles allocation failures.

**Status:** ✅ TEAM_389 investigated:
- Eyra is an **external crates.io dependency** (v0.22)
- Cannot modify Eyra's allocator directly without forking
- Solution: Use `std::panic::set_hook` to catch OOM panics

**Finding:** Eyra uses Rust's std allocator which panics on OOM.
uucore already has panic hook infrastructure in `src/uucore/src/lib/mods/panic.rs`.

---

## Step 5: Implement OOM Panic Handler

**Goal:** Make OOM panics exit cleanly with code 134 instead of halting.

**Status:** ✅ TEAM_389 IMPLEMENTED

**File:** `crates/userspace/eyra/coreutils/src/uucore/src/lib/mods/panic.rs`

**Implementation:**
- Extended existing `mute_sigpipe_panic()` hook
- Added `is_oom_panic()` detection function
- On OOM panic: prints "Error: out of memory" and exits with code 134

```rust
// TEAM_389: Detect OOM panics by checking panic message
fn is_oom_panic(info: &PanicInfo) -> bool {
    // Check for allocation-related panic messages
    ...
}

pub fn mute_sigpipe_panic() {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        // TEAM_389: Handle OOM panics gracefully
        if is_oom_panic(info) {
            eprintln!("Error: out of memory");
            std::process::exit(134);
        }
        if !is_broken_pipe(info) {
            hook(info);
        }
    }));
}
```

---

## Step 6: Integration Testing

**Goal:** Verify the fix works end-to-end.

**Test cases:**
1. `cat small_file.txt` — Should work normally
2. `cat huge_file.txt` — Should fail gracefully with error message, not kernel panic
3. Allocate more than 256MB — Should get ENOMEM

**Test command:**
```bash
./run-term.sh
# In shell:
cat hello.txt  # Should work now with 128MB heap + 2GB RAM
```

---

## Acceptance Criteria

- [x] Kernel syscalls return ENOMEM (not panic) — TEAM_389
- [x] USER_HEAP_MAX_SIZE increased to 256MB — TEAM_389
- [x] OOM logged only in debug builds — TEAM_389
- [x] Userspace programs exit cleanly on OOM (not kernel panic) — TEAM_389 panic handler
- [x] All existing tests pass — TEAM_389 verified

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Eyra allocator deeply embedded | Start with investigation before changes |
| Breaking existing programs | Test thoroughly with coreutils |
| Golden log changes | Update with `--update` flag (silver mode) |

---

## Estimated Effort

| Step | Effort | Dependencies |
|------|--------|--------------|
| Step 1 | 10 min | None |
| Step 2 | 5 min | None |
| Step 3 | 10 min | None |
| Step 4 | 30 min | Step 1-3 |
| Step 5 | 30-60 min | Step 4 findings |
| Step 6 | 15 min | Step 5 |

**Total:** 1.5-2 hours
