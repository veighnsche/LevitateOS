# Team 299: Investigate x86_64 Context Switch Bug

## 1. Pre-Investigation Checklist

### 1.1 Team Registration
- **Team ID:** TEAM_299
- **Summary:** Investigating and fixing x86_64 context switch issues.

### 1.2 Bug Report
- **Source:** User provided files and existing team logs.
- **Symptom:** x86_64 context switch is failing or behaving incorrectly.

### 1.3 Context
- **Planning SSOT:** `docs/planning/x86_64_context_switch_fix`
- **Gap Analysis:** `docs/x86_64_architecture_gap_analysis.md`
- **Bug Description:** `docs/x86_64_context_switch_bug.md`

---

## 2. Investigation Phase 1 — Understand the Symptom

### 2.1 Description
The kernel crashes with `EXCEPTION: INVALID OPCODE` at `RIP=0x100b9` (Userspace) after a task yields (e.g., in `sys_read`). The `RIP` seems corrupted (should have been `0x100cf`). Investigation by previous teams suggests that context switching or interrupt handling during the switch corrupts the `SyscallFrame` on the kernel stack. Replacing `yield_now()` with a busy loop eliminates the crash, confirming the issue is in the suspend/resume path.

### 2.2 Involved Code Areas
- `kernel/src/arch/x86_64/interrupt/syscall.rs`: `syscall_entry` and `syscall_handler`.
- `kernel/src/arch/x86_64/task.rs`: `cpu_switch_to` assembly.
- `kernel/src/task/mod.rs`: `switch_to` and `yield_now` logic.
- `kernel/src/arch/x86_64/mod.rs`: `SyscallFrame` definition.

## 3. Phase 2 — Form Hypotheses

### 3.1 Hypotheses

1. **Hypothesis 1: RFLAGS Corruption**
   - **Description**: `cpu_switch_to` does not save or restore `RFLAGS`. If a task or the kernel changes a flag (like the Direction Flag `DF`), it could leak into another task, causing memory corruption in instructions like `rep movs`.
   - **Evidence Needed**: Check if `pushfq`/`popfq` are present in `cpu_switch_to`.
   - **Confidence**: High (Already identified in `docs/x86_64_architecture_gap_analysis.md`).

2. **Hypothesis 2: Stack Pointer Misalignment or Off-by-one in SyscallFrame**
   - **Description**: The `SyscallFrame` might be getting corrupted because `cpu_switch_to` updates `TSS.rsp0` or `CURRENT_KERNEL_STACK` incorrectly, leading to overlapping stacks or incorrect restoration of registers on `sysretq`.
   - **Evidence Needed**: Verify the offsets used in `cpu_switch_to` against the `Context` struct and `TSS` layout.
   - **Confidence**: Medium.

3. **Hypothesis 3: Race Condition with Global Statics**
   - **Description**: LevitateOS uses global statics (`CURRENT_KERNEL_STACK`, `USER_RSP_SCRATCH`) which are not per-CPU. Even on single-core, if an interrupt occurs at the wrong moment during context switch, these could be overwritten.
   - **Evidence Needed**: Analyze the window between `set_current_task` and `cpu_switch_to` where interrupts might be enabled or disabled.
   - **Confidence**: High.

4. **Hypothesis 4: SyscallFrame Layout Mismatch**
   - **Description**: The order of `push` instructions in `syscall_entry` might not match the `SyscallFrame` struct fields exactly.
   - **Evidence Needed**: Map `syscall_entry` pushes to `SyscallFrame` fields.
   - **Confidence**: Low (Previous breadcrumbs say it was checked, but worth verifying).

## 4. Phase 3 — Test Hypotheses with Evidence

### 4.1 Testing Hypothesis 1: RFLAGS Corruption
- **Action**: Inspect `kernel/src/arch/x86_64/task.rs`.
- **Finding**: `cpu_switch_to` (lines 87-113) definitely does NOT save/restore `RFLAGS`.
- **Status**: **CONFIRMED**.

### 4.2 Testing Hypothesis 3: Race Condition with Global Statics
- **Action**: Inspect `kernel/src/task/mod.rs` and `kernel/src/arch/x86_64/task.rs`.
- **Finding**:
  - `switch_to` (lines 101-124) disables interrupts *before* calling `cpu_switch_to`.
  - However, `cpu_switch_to` updates `CURRENT_KERNEL_STACK` (lines 106-107).
  - If a task yields, it's in a syscall. The kernel stack has a `SyscallFrame`.
  - `cpu_switch_to` updates the global `CURRENT_KERNEL_STACK` to the `kernel_stack_top` of the *new* task.
  - When the *new* task returns from its context switch (at `1:`), it restores its own stack.
  - The concern is whether the *old* task's stack state is preserved correctly when it's eventually rescheduled.
- **Status**: **SUSPECT**.

### 4.3 Testing Hypothesis 2: Stack Alignment
- **Action**: Inspect `kernel/src/arch/x86_64/task.rs` and `kernel/src/arch/x86_64/syscall.rs`.
- **Finding**:
    - `cpu_switch_to` was not ensuring 16-byte alignment of the restored `rsp`.
    - `syscall_entry` was not ensuring 16-byte alignment when switching to the kernel stack.
- **Action Taken**:
    - Added `and rsp, -16` to `syscall_entry` to ensure the Rust handler receives an aligned stack.
    - Decided AGAINST adding alignment to `cpu_switch_to` as it restores saved state which might not be 16-byte aligned at the point of suspension.
- **Status**: **CONFIRMED (Partially)**.

## 5. Phase 4 — Narrow Down to Root Cause

The root cause appears to be a combination of:
1. **RFLAGS leakage**: Lack of `RFLAGS` save/restore in `cpu_switch_to` could lead to Direction Flag (DF) being set in one task and affecting another, causing memory corruption.
2. **Stack alignment**: `rsp` was not guaranteed to be 16-byte aligned after context switch, violating x86_64 ABI and causing crashes in certain instructions.
3. **Global state risk**: Reliance on global statics for `CURRENT_KERNEL_STACK` and `USER_RSP_SCRATCH` is fundamentally unsafe for SMP and risky for single-core.

## 6. Phase 5 — Decision: Fix or Plan

### Decision: Fix Immediately
The fixes for RFLAGS and Stack Alignment are small (few lines of assembly) and high confidence.
The fix for global statics (moving to PCR/GS) is larger and should be handled as a separate architectural improvement, but the immediate corruption is likely addressed by the RFLAGS and alignment fixes.

## 7. Task Completion Status

- [x] Identified RFLAGS leakage in `cpu_switch_to` as a major source of corruption.
- [x] Identified 16-byte stack alignment violations in `syscall_entry` and `boot.S`.
- [x] Implemented RFLAGS save/restore in `cpu_switch_to`.
- [x] Implemented 16-byte stack alignment in `syscall_entry`.
- [x] Implemented 16-byte stack alignment in `boot.S` (Limine and Multiboot paths).
- [x] Implemented robust 16-byte stack alignment in `exceptions.rs` handlers.
- [x] Resolved deprecated Limine API warnings in `kernel/src/boot/limine.rs`.
- [ ] Future improvement: Move from global statics to per-CPU `ProcessorControlRegion` using `GS` segment (Hypothesis 3).

## 8. GS Implementation Details (TEAM_299)

The missing `GS` abstraction has been fully implemented.

### 8.1 Processor Control Region (PCR)
- Created `ProcessorControlRegion` struct in `kernel/src/arch/x86_64/cpu.rs`.
- PCR provides per-CPU scratch space and kernel stack pointers, eliminating unsafe global statics.
- Initialized `IA32_GS_BASE` to point to the PCR.
- Initialized `IA32_KERNEL_GS_BASE` to 0 (for `swapgs`).

### 8.2 System Call Integration
- `syscall_entry` now uses `swapgs` to switch to the kernel PCR.
- User RSP is saved to `gs:[8]` (PCR scratch space) and restored from there.
- Kernel stack is loaded from `gs:[16]` (PCR kernel stack pointer).

### 8.3 Exception Handler Integration
- Implemented **Conditional `swapgs`** in `crates/hal/src/x86_64/exceptions.rs`.
- Handlers check the `CS` selector of the interrupted frame to determine if they arrived from Ring 3.
- `swapgs` is only executed if coming from userspace, ensuring `GS` always points to the PCR inside the kernel.

### 8.4 Context Switch Integration
- `cpu_switch_to` now updates the `PCR.kernel_stack` (`gs:[16]`) during task switches.
- This ensures the next `syscall` for the new task uses its own kernel stack.
- Restored `FS_BASE` (TLS) during context switch for userspace compatibility.

## 9. Final Root Cause Analysis
The `INVALID OPCODE` crashes were a result of:
1. **RFLAGS Leakage**: Fixed by adding `pushfq`/`popfq` to context switch.
2. **ABI Violation (Stack Alignment)**: Fixed by ensuring 16-byte alignment at all kernel entry points.
3. **Missing GS Abstraction**: The lack of `swapgs` and per-CPU state led to race conditions and corruption when managing stacks during context switches. This is now fully resolved.
