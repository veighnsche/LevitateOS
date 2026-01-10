# Phase 2: Root Cause Analysis
**Team**: TEAM_298
**Bug**: x86_64 Context Switch & Syscall Corruption

---

## 1. Hypotheses & Evidence

### H1: RFLAGS Corruption (Leading Theory)
**Theory**: `cpu_switch_to` does not save/restore the RFLAGS register.
**Evidence**:
- Code inspection of `kernel/src/arch/x86_64/task.rs` confirms RFLAGS is NOT saved.
- **Why it matters**: The System V AMD64 ABI requires the Direction Flag (DF) to be cleared (0) on function entry. If a task or interrupt handler sets DF=1 (for backwards copy) and then yields, the next task running on that CPU will wake up with DF=1.
- **Impact**: Code relying on `std` (like `memcpy` using `rep movsb`) will copy backwards, corrupting memory. The `SyscallFrame` could be corrupted this way.

### H2: Global Variable Race (Confirmed Arch Flaw)
**Theory**: Using global `CURRENT_KERNEL_STACK` instead of per-CPU state causes context switch logic to be fragile.
**Evidence**:
- Comparison with Redox Kernel shows LevitateOS uses globals where Redox uses GS-relative per-CPU data.
- While incorrect for SMP, on a single core this *should* work mostly, provided interrupts/exceptions don't clobber the global while it's "borrowed" by a task.
- **Status**: Confirmed architectural issue, but likely H1 is the immediate cause of the hard crash.

---

## 2. Key Code Areas

- `kernel/src/arch/x86_64/task.rs`: `cpu_switch_to` assembly.
- `kernel/src/arch/x86_64/syscall.rs`: Syscall entry/exit logic.

---

## 3. Investigation Conclusion

The missing RFLAGS save/restore is a critical violation of x86_64 context switching requirements. The global state usage is a long-term risk but fixing RFLAGS is the minimal effective change to restore stability.

**Root Cause**: `cpu_switch_to` fails to preserve RFLAGS, leading to register state leak between tasks (Likely DF flag).
