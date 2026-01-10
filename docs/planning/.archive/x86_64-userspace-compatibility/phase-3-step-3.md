# Phase 3 — Step 3: Context Switching and User Entry

## Parent
[Phase 3: Implementation](phase-3.md)

## Goal
Implement the core architecture stubs for task management and user-mode transition.

## Tasks
1. **Implement `cpu_switch_to`**:
   - Write assembly to save/restore callee-saved registers (RBX, RBP, R12-R15).
   - Swap stack pointers.
2. **Implement `enter_user_mode`**:
   - In `kernel/src/arch/x86_64/task.rs`, implement the transition to Ring 3.
   - Use `iretq` or `sysretq` to jump to user RIP and set user RSP.
   - Ensure GDT selectors for User Code and User Data are correct.
3. **Implement `task_entry_trampoline`**:
   - Provide the initial jump point for new kernel tasks.

## Exit Criteria
- Kernel can switch between tasks on x86_64.
- `enter_user_mode` successfully transitions to Ring 3.
