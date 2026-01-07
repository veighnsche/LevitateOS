# Phase 3: Implementation - HIGH Priority Memory TODOs

**Feature:** TODO Cleanup & Crate Audit  
**Team:** TEAM_235  
**Status:** READY FOR IMPLEMENTATION

---

## Overview

This phase implements the 3 HIGH priority memory management TODOs identified in Phase 1.

### Scope (Per Q6 Decision)
- **In Scope:** HIGH priority items only (memory safety)
- **Out of Scope:** MEDIUM/LOW items (tracked as issues for future work)

### Implementation Order
1. **Step 1:** `destroy_user_page_table` - Foundation for cleanup
2. **Step 2:** mmap failure cleanup - Uses RAII pattern
3. **Step 3:** VMA tracking - Enables proper munmap

---

## Step 1: Page Table Teardown (`destroy_user_page_table`)

**Location:** `kernel/src/memory/user.rs`  
**Current State:** No-op stub that leaks pages  
**Goal:** Properly free all pages and page tables when process exits

### UoW Breakdown

#### UoW 1.1: Add recursive page table walker helper

**File:** `kernel/src/memory/user.rs`

**Task:** Create a helper function that walks page tables recursively.

```rust
/// Walk page table at given level and call visitor for each valid entry.
/// Returns list of table physical addresses for later freeing.
unsafe fn walk_page_table_level(
    table_phys: usize,
    level: usize,
    visitor: &mut dyn FnMut(usize, usize), // (phys_addr, level)
) -> Vec<usize>
```

**Acceptance Criteria:**
- Function compiles
- Handles L0 → L1 → L2 → L3 traversal
- Distinguishes block entries from table entries
- Returns list of intermediate table addresses

---

#### UoW 1.2: Implement leaf page freeing

**File:** `kernel/src/memory/user.rs`

**Task:** In the walker, free leaf pages (L3 entries and block mappings).

**Changes:**
1. When visitor encounters L3 page entry → call `FRAME_ALLOCATOR.free_page(phys)`
2. When visitor encounters L1/L2 block entry → call appropriate free

**Acceptance Criteria:**
- L3 page entries are freed
- 2MB block entries (if any) are freed
- No double-free (check entry validity first)

---

#### UoW 1.3: Implement table freeing (bottom-up)

**File:** `kernel/src/memory/user.rs`

**Task:** After all leaves are freed, free the intermediate tables bottom-up.

**Changes:**
1. Collect table addresses during walk
2. Free in reverse order (L3 tables, then L2, then L1, then L0)

**Acceptance Criteria:**
- All intermediate page tables freed
- L0 table itself freed last
- TLB flushed after teardown

---

#### UoW 1.4: Wire up destroy_user_page_table

**File:** `kernel/src/memory/user.rs`

**Task:** Replace the stub with actual implementation.

**Changes:**
```rust
pub unsafe fn destroy_user_page_table(ttbr0_phys: usize) -> Result<(), MmuError> {
    // 1. Walk and free all pages
    // 2. Free all tables bottom-up
    // 3. Flush TLB
    Ok(())
}
```

**Acceptance Criteria:**
- Function no longer returns immediately
- All pages and tables freed
- No memory leaks on process exit

---

#### UoW 1.5: Add unit test for page table teardown

**File:** `kernel/src/memory/user.rs` (test module)

**Task:** Add test that creates a user page table, maps some pages, then destroys it.

**Test outline:**
1. Create user page table
2. Map a few pages at different addresses
3. Call destroy_user_page_table
4. Verify no panic, no leak (if allocator stats available)

**Acceptance Criteria:**
- Test compiles and passes
- Test exercises the full teardown path

---

## Step 2: mmap Failure Cleanup

**Location:** `kernel/src/syscall/mm.rs`  
**Current State:** Partial allocations leak on failure  
**Goal:** Clean up all allocated pages if mmap fails partway through

### UoW Breakdown

#### UoW 2.1: Create MmapGuard RAII type

**File:** `kernel/src/syscall/mm.rs`

**Task:** Create an RAII guard that tracks allocated pages and frees them on drop.

```rust
/// RAII guard for mmap allocation cleanup.
/// Tracks pages allocated during mmap. On drop, frees all unless committed.
struct MmapGuard {
    pages: Vec<(usize, usize)>, // (va, phys) pairs
    ttbr0: usize,
    committed: bool,
}

impl MmapGuard {
    fn new(ttbr0: usize) -> Self { ... }
    fn track(&mut self, va: usize, phys: usize) { ... }
    fn commit(mut self) { self.committed = true; }
}

impl Drop for MmapGuard {
    fn drop(&mut self) {
        if !self.committed {
            // Unmap and free all tracked pages
        }
    }
}
```

**Acceptance Criteria:**
- Struct compiles
- Drop implementation frees pages when not committed
- commit() prevents cleanup

---

#### UoW 2.2: Integrate MmapGuard into sys_mmap

**File:** `kernel/src/syscall/mm.rs`

**Task:** Use MmapGuard in the mmap syscall implementation.

**Changes:**
1. Create guard at start of allocation loop
2. Track each allocated page
3. Call `guard.commit()` only on success
4. Remove manual cleanup code (guard handles it)

**Acceptance Criteria:**
- sys_mmap uses MmapGuard
- Failure at any point cleans up prior allocations
- Success path commits the guard

---

#### UoW 2.3: Add test for mmap failure cleanup

**File:** `kernel/src/syscall/mm.rs` (test module or integration test)

**Task:** Test that simulates mmap failure and verifies cleanup.

**Test outline:**
1. Set up scenario where mmap will fail partway (e.g., OOM)
2. Call sys_mmap
3. Verify pages were freed (no leak)

**Acceptance Criteria:**
- Test documents the expected behavior
- Test passes (may need mock allocator for OOM simulation)

---

## Step 3: VMA Tracking for munmap

**Location:** New file `kernel/src/memory/vma.rs` + updates to `task/mod.rs`  
**Current State:** No VMA tracking, munmap is stub  
**Goal:** Track mapped regions to enable proper unmapping

### UoW Breakdown

#### UoW 3.1: Create VMA types

**File:** `kernel/src/memory/vma.rs` (new file)

**Task:** Define the VMA data structures.

```rust
//! Virtual Memory Area tracking for user processes.

use bitflags::bitflags;

bitflags! {
    /// VMA permission flags.
    pub struct VmaFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC = 1 << 2;
    }
}

/// A contiguous virtual memory region.
#[derive(Debug, Clone)]
pub struct Vma {
    pub start: usize,
    pub end: usize,
    pub flags: VmaFlags,
}

impl Vma {
    pub fn new(start: usize, end: usize, flags: VmaFlags) -> Self { ... }
    pub fn len(&self) -> usize { self.end - self.start }
    pub fn contains(&self, addr: usize) -> bool { ... }
    pub fn overlaps(&self, start: usize, end: usize) -> bool { ... }
}
```

**Acceptance Criteria:**
- File compiles
- VmaFlags has READ/WRITE/EXEC
- Vma has start/end/flags and helper methods

---

#### UoW 3.2: Create VmaList container

**File:** `kernel/src/memory/vma.rs`

**Task:** Create a container to manage VMAs for a process.

```rust
/// List of VMAs for a process.
#[derive(Debug, Default)]
pub struct VmaList {
    vmas: Vec<Vma>,
}

impl VmaList {
    pub fn new() -> Self { ... }
    
    /// Add a new VMA. Returns error if overlaps existing.
    pub fn insert(&mut self, vma: Vma) -> Result<(), VmaError> { ... }
    
    /// Remove VMA(s) covering the given range.
    pub fn remove(&mut self, start: usize, end: usize) -> Result<Vec<Vma>, VmaError> { ... }
    
    /// Find VMA containing address.
    pub fn find(&self, addr: usize) -> Option<&Vma> { ... }
    
    /// Find VMAs overlapping range.
    pub fn find_overlapping(&self, start: usize, end: usize) -> Vec<&Vma> { ... }
}
```

**Acceptance Criteria:**
- VmaList compiles
- insert() rejects overlapping VMAs
- remove() handles partial overlaps (splits VMAs)
- find() returns correct VMA

---

#### UoW 3.3: Add VmaList to TaskControlBlock

**File:** `kernel/src/task/mod.rs`

**Task:** Add VMA tracking to each process.

**Changes:**
1. Add `vmas: IrqSafeLock<VmaList>` field to TaskControlBlock
2. Initialize empty VmaList in constructors
3. Export vma module from memory

**Acceptance Criteria:**
- TCB has vmas field
- Field initialized in all constructors
- No compilation errors

---

#### UoW 3.4: Update sys_mmap to record VMAs

**File:** `kernel/src/syscall/mm.rs`

**Task:** When mmap succeeds, record the VMA.

**Changes:**
1. After successful mapping, create Vma struct
2. Insert into task's VmaList
3. Handle insert failure (shouldn't happen if we check first)

**Acceptance Criteria:**
- Successful mmap creates VMA entry
- VMA has correct start/end/flags

---

#### UoW 3.5: Implement sys_munmap using VMA

**File:** `kernel/src/syscall/mm.rs`

**Task:** Implement actual munmap using VMA tracking.

**Changes:**
1. Look up VMA(s) covering the range
2. For each page in range: unmap and free
3. Update/remove VMA entries
4. Return success

```rust
pub fn sys_munmap(addr: usize, len: usize) -> i64 {
    let task = current_task();
    let mut vmas = task.vmas.lock();
    
    // 1. Find and remove VMAs in range
    // 2. Unmap pages
    // 3. Free physical pages
    // 4. Return 0 on success
}
```

**Acceptance Criteria:**
- munmap finds correct VMAs
- Pages are unmapped and freed
- VMA list updated correctly

---

#### UoW 3.6: Add unit tests for VMA operations

**File:** `kernel/src/memory/vma.rs` (test module)

**Task:** Test VMA insert, remove, find, and overlap handling.

**Tests:**
1. Insert non-overlapping VMAs
2. Reject overlapping insert
3. Remove exact VMA
4. Remove partial (split VMA)
5. Find by address

**Acceptance Criteria:**
- All tests pass
- Edge cases covered (adjacent VMAs, partial overlaps)

---

## Verification Checklist

After all UoWs complete:

- [ ] `destroy_user_page_table` frees all pages
- [ ] mmap failure cleans up partial allocations
- [ ] munmap properly unmaps and frees
- [ ] All existing tests pass
- [ ] Golden boot test passes
- [ ] No memory leaks (verify with allocator stats if available)

---

## Dependencies

| UoW | Depends On |
|-----|------------|
| 1.2 | 1.1 |
| 1.3 | 1.2 |
| 1.4 | 1.3 |
| 1.5 | 1.4 |
| 2.2 | 2.1 |
| 2.3 | 2.2 |
| 3.2 | 3.1 |
| 3.3 | 3.2 |
| 3.4 | 3.3, 2.2 |
| 3.5 | 3.4 |
| 3.6 | 3.2 |

---

## Estimated Effort

| Step | UoWs | Complexity |
|------|------|------------|
| Step 1 (Page Teardown) | 5 | Medium |
| Step 2 (mmap Cleanup) | 3 | Low |
| Step 3 (VMA Tracking) | 6 | Medium |
| **Total** | **14** | |

Each UoW is designed to be completable by an SLM in a single session (~50-200 lines of code each).
