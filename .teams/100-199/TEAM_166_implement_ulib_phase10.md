# TEAM_166: Implement ulib Phase 10

## Objective
Implement Phase 10 (Userspace Standard Library) following the approved plan.

## Context
- **Plan:** `docs/planning/ulib-phase10/phase-3.md` (READY FOR IMPLEMENTATION)
- **Reviewed by:** TEAM_165
- **Decisions:** See phase-2.md and phase-3.md decision summary

## Implementation Progress

### Step 1: Kernel sbrk ✅
- [x] UoW 1: Add ProcessHeap struct
  - Added `ProcessHeap` struct to `kernel/src/task/user.rs`
  - Tracks base, current, and max heap addresses
  - Integrated into `UserTask` and `TaskControlBlock`
- [x] UoW 2: Implement sys_sbrk
  - Replaced stub in `kernel/src/syscall.rs`
  - Allocates and maps pages via `alloc_and_map_heap_page()`
  - Returns previous break on success, 0 on OOM (per Q3)

### Step 2: ulib Allocator ✅
- [x] UoW 1: Create ulib crate
  - Created `userspace/ulib/` with Cargo.toml and lib.rs
  - Added to workspace
  - Depends on libsyscall
- [x] UoW 2: Implement global allocator
  - Created `LosAllocator` bump allocator in `alloc.rs`
  - Backed by sbrk syscall
  - `#[global_allocator]` and `#[alloc_error_handler]`

## Status
- COMPLETE

## Files Modified

### Kernel
- `kernel/src/task/user.rs` — Added `ProcessHeap` struct, integrated into `UserTask`
- `kernel/src/task/mod.rs` — Added heap field to `TaskControlBlock`
- `kernel/src/task/user_mm.rs` — Added `alloc_and_map_heap_page()` helper
- `kernel/src/syscall.rs` — Implemented `sys_sbrk`

### Userspace
- `userspace/Cargo.toml` — Added ulib to workspace
- `userspace/ulib/Cargo.toml` — New crate
- `userspace/ulib/src/lib.rs` — Main library entry
- `userspace/ulib/src/alloc.rs` — Global allocator

## Handoff Checklist
- [x] Project builds cleanly
- [x] All tests pass
- [x] Team file updated
- [x] Code comments include TEAM_166

## Next Steps (for future teams)
1. Step 3-4: File syscalls and abstractions
2. Step 5-6: Argument/environment passing
3. Step 7-8: Time syscalls
4. Step 9: Integration demo
