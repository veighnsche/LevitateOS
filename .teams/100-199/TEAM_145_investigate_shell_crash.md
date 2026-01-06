# TEAM_145: Investigate Shell Data Abort Crash

## Status: FIXED ✅

## Bug Report

**Symptom:** Shell prints banner and prompt, then crashes with USER EXCEPTION (Data Abort)

**Exception at:** `strb wzr, [sp, #268]` = initializing `line_len` variable on stack

---

## Root Cause Analysis

### The Bug

TEAM_143's performance optimization added `yield_now()` in `sys_read`:

```rust
if bytes_read == 0 {
    crate::task::yield_now();  // ← BUG: Context switches to another task!
    aarch64_cpu::asm::wfi();
}
```

### Why It Crashes

1. Shell (PID=2, TTBR0=shell_tables) calls `sys_read`
2. No input available, so `bytes_read == 0`
3. `yield_now()` → `yield_and_reschedule()` picks init (PID=1)
4. `switch_to(init)` changes `CURRENT_TASK` but NOT TTBR0
5. Syscall returns via `eret` — but we're returning with **wrong TTBR0**!
6. Shell code runs but TTBR0 still has init's page tables
7. Stack access at 0x7ffffffeff5c hits unmapped region → Translation fault

### TEAM_145 BREADCRUMB: CONFIRMED
**File:** `kernel/src/syscall.rs:265`
**Root cause:** `yield_now()` should NOT be called from syscall handlers because the eret return path does not restore TTBR0.

---

## The Fix

**Remove `yield_now()` from sys_read.** Keep only `wfi()` for power efficiency:

```diff
 if bytes_read == 0 {
-    crate::task::yield_now();
     #[cfg(target_arch = "aarch64")]
     aarch64_cpu::asm::wfi();
 }
```

**Rationale:** `wfi` is sufficient — it waits for keyboard/timer interrupt. When interrupt fires, it wakes the CPU, syscall continues polling, finds input, returns normally.

---

## Files to Modify

- `kernel/src/syscall.rs:265` - Remove yield_now() call
