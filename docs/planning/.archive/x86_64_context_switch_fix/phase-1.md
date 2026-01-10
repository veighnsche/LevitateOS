# Phase 1: Understanding and Scoping
**Team**: TEAM_298
**Bug**: x86_64 Context Switch & Syscall Corruption
**Status**: DRAFT

---

## 1. Bug Summary

**Description**: The kernel panics with `EXCEPTION: INVALID OPCODE` at `RIP=0x100b9` (userspace shell) immediately after the shell prompt appears. The crash occurs upon returning from a syscall (likely `sys_read`) after a context switch.

**Severity**: **Critical**. It prevents reliable userspace execution on x86_64, specifically for interactive shells or any process that yields.

**Impact**:
- Users cannot interact with the shell.
- The OS is unstable on x86_64.
- Blocking syscalls cause immediate crashes upon return.

---

## 2. Reproduction Status

**Reproducible?**: Yes, 100%.

**Steps**:
1. Build and run the x86_64 kernel with userspace.
   ```bash
   cargo xtask run term --arch x86_64
   ```
2. Wait for the shell prompt `/>`.
3. The kernel immediately panics with `INVALID OPCODE`.

**Expected**: The shell waits for user input without crashing.
**Actual**: Crash at `RIP=0x100b9` (inside a `call` instruction, offset by -3 bytes vs expected return).

---

## 3. Context & Root Cause Analysis (from TEAM_298 investigation)

**Investigation Findings**:
- **Code Areas**: `kernel/src/arch/x86_64/syscall.rs`, `kernel/src/arch/x86_64/task.rs`, `kernel/src/task/mod.rs`.
- **Mechanism**: The crash is caused by memory corruption of the `SyscallFrame` (specifically `RCX`) on the kernel stack during task switching.
- **Root Cause**:
    1. **Missing Per-CPU State**: LevitateOS uses global `static mut` variables (`CURRENT_KERNEL_STACK`, `USER_RSP_SCRATCH`) instead of per-CPU storage (like `GS` segment). This is architecturally unsound for x86_64.
    2. **Missing RFLAGS Preservation**: The `cpu_switch_to` assembly function saves general-purpose registers but **fails to save/restore RFLAGS**. This can lead to the Direction Flag (DF) being incorrect after a switch, causing `rep movsb` operations (used in `memcpy`) to corrupt memory.

**Previous Team Findings**:
- **TEAM_296**: Identified crash site.
- **TEAM_297**: Ruled out GOT/PLT issues. Confirmed `yield_now()` is the trigger.
- **TEAM_298**: Detailed architectural gap analysis vs Redox. Confirmed lack of RFLAGS save/restore and unsafe use of globals.

---

## 4. Constraints

- **Time**: High urgency.
- **Stability**: Must work for the single-core case immediately. SMP support is a long-term goal but the fix should move us towards it or at least not block it.
- **Backward Compatibility**: Must not break AArch64 (though this is x86_64 specific code).

---

## 5. Open Questions

None currently. The RFLAGS hypothesis is strong enough to warranty a fix attempt. The global variable architectural flaw is confirmed and requires a longer-term fix, but the RFLAGS issue is likely the immediate crasher.
