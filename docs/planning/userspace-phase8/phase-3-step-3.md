# Phase 3 / Step 3: User Address Space

## Goal
Implement per-process TTBR0 page tables and user memory management.

## Parent Context
- [Phase 3](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3.md)
- [Phase 2 Design](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md)

## Design Reference
From Phase 2:
- Each process gets its own TTBR0
- User pages mapped with `AP_RW_EL0` flags
- Address space layout: code, data, heap, stack
- TTBR1 (kernel) remains shared

## Units of Work

### UoW 1: User Page Flags
Add page flag constants for user-mode mappings.

**Tasks:**
1. Modify `levitate-hal/src/mmu.rs`.
2. Add `AP_RW_EL0` flag constant (user-accessible).
3. Add `USER_DATA` and `USER_CODE` flag presets.
4. Ensure user pages have `UXN` bit set for data (non-executable in kernel).

**Exit Criteria:**
- `PageFlags::USER_DATA` and `PageFlags::USER_CODE` available.
- Flags correctly set for EL0 access.

### UoW 2: User Page Table Creation
Implement per-process TTBR0 allocation.

**Tasks:**
1. Create `kernel/src/task/user_mm.rs`.
2. Implement `create_user_page_table() -> *mut PageTable`.
3. Allocate L0 table from buddy allocator.
4. Return physical address (for TTBR0 register).

**Exit Criteria:**
- Can allocate a fresh L0 page table.
- Page table is zeroed and ready for mappings.

### UoW 3: User Memory Mapping
Implement mapping functions for user space.

**Tasks:**
1. Implement `map_user_page(ttbr0, va, pa, flags)`.
2. Implement `map_user_range(ttbr0, va_start, pa_start, len, flags)`.
3. Use `walk_to_entry` with `create = true`.
4. Ensure page tables are allocated from buddy allocator.

**Exit Criteria:**
- Can map arbitrary user pages.
- Pages are accessible from EL0.

### UoW 4: TTBR0 Switching
Switch TTBR0 on context switch.

**Tasks:**
1. Modify `cpu_switch_to` or `switch_to` to update TTBR0.
2. When switching to user task, load its TTBR0.
3. When switching to kernel task, restore kernel TTBR0 (or leave unchanged).
4. TLB invalidation after TTBR0 switch.

**Exit Criteria:**
- Context switch correctly updates TTBR0.
- No TLB stale entries between processes.

## Expected Outputs
- User page flags in `mmu.rs`.
- `kernel/src/task/user_mm.rs` with user memory management.
- TTBR0 switching in context switch path.
