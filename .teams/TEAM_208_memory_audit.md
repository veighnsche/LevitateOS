# Team 208 - Memory Management Audit

## Status
- [x] Claim team number
- [ ] Audit codebase for scattered memory management code
- [ ] Analyze findings and propose migration plan

## Context
The user is concerned that `kernel/src/memory/mod.rs` is too small (250 LOC) for an entire OS, suggesting memory management logic might be scattered across the codebase.

## Findings
The memory management logic is currently partitioned across several layers:

1.  **`los-hal` (Crate):** Contains the core architecture-specific implementation (`mmu.rs`) and generic allocators (`allocator/`).
2.  **`kernel/src/memory/mod.rs`:** Focuses on physical memory discovery (FDT), `FRAME_ALLOCATOR` initialization, and mapping the initial memory layout.
3.  **`kernel/src/task/user_mm.rs`:** Handles user-space specific memory logic (page table creation, stack/heap allocation, buffer validation).
4.  **`kernel/src/task/user.rs`:** Defines `ProcessHeap` and heap management structures.
5.  **`kernel/src/syscall/mm.rs`:** Implements memory-related system calls like `sbrk`.

### Recommendations
While the code is functional, `user_mm.rs` and `ProcessHeap` are logically "memory management" rather than "task management". 

- **Phase 1:** Move `kernel/src/task/user_mm.rs` to `kernel/src/memory/user.rs`.
- **Phase 2:** Move `ProcessHeap` from `kernel/src/task/user.rs` to `kernel/src/memory/heap.rs` (or similar).
- **Phase 3:** Ensure `kernel/src/memory/mod.rs` acts as the central entry point for all memory-related kernel logic.
