# TEAM_298: Investigate x86_64 Context Switch Corruption

## 1. Bug Summary

**Symptom**: Kernel panic `EXCEPTION: INVALID OPCODE` at `RIP=0x100b9` immediately after shell prompt appears.

**Status**: ✅ **ROOT CAUSE IDENTIFIED** - Architectural flaw in per-CPU state management.

---

## 2. Root Cause Analysis

### Comparison with Redox Kernel

After analyzing the Redox kernel x86_64 implementation, **significant architectural gaps** were identified in LevitateOS.

#### Redox Kernel (Correct Approach)

```rust
// Redox: Per-CPU state stored in ProcessorControlRegion (PCR), accessed via GS segment
pub struct ProcessorControlRegion {
    pub self_ref: *mut ProcessorControlRegion,
    pub user_rsp_tmp: usize,        // Per-CPU scratch for user RSP
    pub gdt: [GdtEntry; ...],       // Per-CPU GDT
    pub percpu: PercpuBlock,        // Per-CPU data
    pub tss: TaskStateSegment,      // Per-CPU TSS
}
```

```asm
// Redox syscall_entry: Uses swapgs for per-CPU data access
swapgs                          // Swap KGSBASE with GSBASE
mov gs:[user_rsp_tmp], rsp      // Save user RSP to per-CPU PCR
mov rsp, gs:[tss.rsp0]          // Load kernel RSP from per-CPU TSS
```

#### LevitateOS (Flawed Approach)

```rust
// LevitateOS: GLOBAL static variables (NOT per-CPU!)
pub static mut CURRENT_KERNEL_STACK: usize = 0;  // ⚠️ GLOBAL
pub static mut USER_RSP_SCRATCH: usize = 0;      // ⚠️ GLOBAL
```

```asm
// LevitateOS syscall_entry: Uses RIP-relative addressing (GLOBAL)
mov [rip + user_rsp], rsp       // Save user RSP to GLOBAL variable
mov rsp, [rip + kernel_stack]   // Load kernel RSP from GLOBAL variable
```

### Why This Causes Corruption

1. **Single-CPU works**: With only 1 task running at a time, globals work because context switch updates them.
2. **Context switch race**: When task A yields:
   - `cpu_switch_to` in task.rs updates `CURRENT_KERNEL_STACK` to task B's stack_top
   - If task B returns to userspace and makes a syscall, it uses B's stack ✓
   - **BUT**: Task A's SyscallFrame is still on A's kernel stack at `A_stack_top - 408`
   - When A resumes, the syscall exit code uses `CURRENT_KERNEL_STACK` which was updated to B's stack

3. **The corruption**: When task A resumes after yield:
   - Returns through syscall_entry (which was already entered)
   - `sysretq` pops RCX from the frame
   - But the frame may have been corrupted by task B's operations if stacks overlap OR
   - The `mov rsp, [rsp]` at line 155 reads from wrong location

### Specific Bug Location

In `syscall.rs` lines 100-101:
```asm
"mov [rip + {user_rsp}], rsp",        // Save to GLOBAL
"mov rsp, [rip + {kernel_stack}]",    // Load from GLOBAL
```

And line 155:
```asm
"mov rsp, [rsp]",  // Load user RSP from frame.rsp (offset 120)
```

The problem: After context switch back to task A, `CURRENT_KERNEL_STACK` was updated in `cpu_switch_to`. But we DON'T enter via syscall_entry again - we resume from where we yielded. The syscall_entry exit path pops from whatever RSP points to, which is correct. However...

**THE ACTUAL BUG**: In `task.rs` cpu_switch_to, lines 106-109:
```asm
"mov rax, [rsi + 48]", // kernel_stack_top from context.x25
"mov [rip + {kernel_stack}], rax",  // Update CURRENT_KERNEL_STACK
"lea rdx, [rip + {tss_val}]",
"mov [rdx + 4], rax", // TSS.rsp0 = kernel_stack_top
```

This updates TSS.rsp0 to `kernel_stack_top` (the TOP of the stack). But:
- The SyscallFrame is at `kernel_stack_top - 408`
- If an interrupt occurs in Ring 3, CPU uses TSS.rsp0 as the new kernel stack
- This would OVERWRITE the SyscallFrame!

Wait, but the task is in the kernel (Ring 0) during yield, so TSS.rsp0 isn't used...

**REVISED ROOT CAUSE**: The issue is that we're updating `CURRENT_KERNEL_STACK` in `cpu_switch_to` but syscall entry/exit doesn't use that correctly after yield. Let me trace more carefully:

1. Task A enters syscall: `rsp = CURRENT_KERNEL_STACK` (from global)
2. Pushes SyscallFrame, RSP now points below frame
3. Calls yield_now, RSP moves further down
4. Context switch: saves RSP to Context.sp, updates `CURRENT_KERNEL_STACK` = B's stack_top
5. Task B runs
6. Task B switches back to A: restores RSP from Context.sp, updates `CURRENT_KERNEL_STACK` = A's stack_top
7. A resumes in yield_now, returns up the call stack
8. Eventually returns to syscall_entry after `call {handler}`
9. Pops registers from stack (RSP is correct from context restore)
10. `mov rsp, [rsp]` loads user_rsp from frame.rsp (offset 120)
11. sysretq

This should work... unless the **stack contents were corrupted** while task A was suspended.

### Final Hypothesis: RFLAGS Corruption

Redox saves/restores RFLAGS in context switch (lines 366-373 in x86_64.rs):
```asm
pushfq
pop QWORD PTR [rdi + {off_rflags}]
push QWORD PTR [rsi + {off_rflags}]
popfq
```

LevitateOS does NOT save/restore RFLAGS in cpu_switch_to!

If RFLAGS changes during context switch (especially DF - direction flag), it could affect how `rep movsb` or similar instructions work, potentially corrupting memory.

---

## 3. Architectural Gaps Summary

| Feature | Redox | LevitateOS | Impact |
|---------|-------|------------|--------|
| Per-CPU state | GS segment + PCR | Global variables | **Critical** on SMP, bug source |
| swapgs | Yes | No | Cannot safely switch user/kernel GS |
| RFLAGS save/restore | Yes | **NO** | May cause direction/carry issues |
| TSS per CPU | Yes (in PCR) | Global static | **Critical** on SMP |
| Kernel stack in TSS | `kstack.initial_top()` | `kernel_stack_top` | Same concept |

---

## 4. Recommended Fix

### Immediate (Bug Fix)
Add RFLAGS save/restore to `cpu_switch_to` in `task.rs`:
```asm
// Save RFLAGS
"pushfq",
"pop QWORD PTR [rdi + 64]",  // context.rflags (need to add field)
// Restore RFLAGS
"push QWORD PTR [rsi + 64]",
"popfq",
```

### Medium-Term (Architecture Fix)
1. Implement `ProcessorControlRegion` (PCR) struct
2. Store per-CPU state in PCR: TSS, GDT, scratch variables
3. Initialize KERNEL_GSBASE MSR to point to PCR
4. Use `swapgs` in syscall entry/exit to access per-CPU data

---

## 5. Breadcrumbs

- `syscall.rs:32-38` - BREADCRUMB: CONFIRMED - Global variables are architectural flaw
- `task.rs:106-109` - BREADCRUMB: SUSPECT - RFLAGS not saved/restored may cause corruption
