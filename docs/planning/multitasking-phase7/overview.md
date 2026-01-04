# Phase 7: Multitasking & Scheduler — Overview

## Feature Summary

**Phase 7** introduces multi-tasking to LevitateOS. Currently, the kernel runs as a single execution flow in `kmain`. This phase will allow the kernel to manage multiple independent paths of execution (tasks).

### Key Components

1. **Virtual Memory Reclamation (`unmap_page`)**
   - Essential for freeing memory when a task terminates.
   - Requires walking page tables and clearing entries.
   - Requires TLB invalidation.

2. **Context Switching**
   - The mechanism to save the state of the current task and restore the state of the next task.
   - Handled in assembly (saving GPRs, SP, ELR).

3. **Scheduler**
   - The logic that decides which task runs next.
   - Initially a simple cooperative Round-Robin, then preemptive using the Generic Timer.

4. **Task Lifecycle Management**
   - Creation, execution, and cleanup of tasks.

---

## Technical Strategy

### 1. `unmap_page()` Implementation
- **Location**: `levitate-hal/src/mmu.rs`
- **Mechanism**: 
  - Walk the page table to the target leaf entry.
  - Assert the entry is valid.
  - Clear the entry.
  - Invalidate the TLB for the virtual address.
  - (Optimization) Recurse upwards to free intermediate page tables if they become empty (reference counting or zero-scanning).

### 2. Context Switching (AArch64)
- **Registers to saved**: x19-x30 (callee-saved), SP_EL0 (if in user), SP_EL1, SPSR_EL1, ELR_EL1.
- **Function**: `fn cpu_switch_to(prev: *mut Context, next: *const Context)`

### 3. Task Struct
```rust
pub struct Task {
    tid: TaskId,
    state: TaskState,
    context: Context,
    stack: Stack,
    page_table: RootPageTable,
}
```

---

## Recommended Sequence

1. **Task 7.1**: Implement `unmap_page` and TLB flushing.
2. **Task 7.2**: Basic Task struct and "Manual" context switch between two hardcoded kernel tasks.
3. **Task 7.3**: Integrated Scheduler and Timer interrupts for preemption.

---

## Current Status: Discovery (Task 7.1)

See `task-7.1-unmap-page.md` for specific implementation details.
