# UoW 2.1: Create MmapGuard RAII Type

**Parent:** Phase 3, Step 2 (mmap Failure Cleanup)  
**File:** `kernel/src/syscall/mm.rs`  
**Complexity:** Low

---

## Context

Currently, if mmap fails partway through allocation, previously allocated pages leak. We need an RAII guard to automatically clean up on failure.

**Problem location:**
```rust
// kernel/src/syscall/mm.rs:128-142
// TODO: Unmap previously allocated pages on failure
// TODO: Free physical pages and unmap on failure
```

---

## Task

Create an RAII guard struct that tracks allocated pages and frees them on drop (unless committed).

---

## Implementation

Add this struct and impl **before** `sys_mmap` in `kernel/src/syscall/mm.rs`:

```rust
/// TEAM_235: RAII guard for mmap allocation cleanup.
///
/// Tracks pages allocated during mmap. On drop, frees all unless committed.
/// This ensures partial allocations are cleaned up on failure.
struct MmapGuard {
    /// (virtual_address, physical_address) pairs
    allocated: alloc::vec::Vec<(usize, usize)>,
    /// User page table physical address
    ttbr0: usize,
    /// Set to true when allocation succeeds
    committed: bool,
}

impl MmapGuard {
    /// Create a new guard for the given user page table.
    fn new(ttbr0: usize) -> Self {
        Self {
            allocated: alloc::vec::Vec::new(),
            ttbr0,
            committed: false,
        }
    }
    
    /// Track an allocated page.
    fn track(&mut self, va: usize, phys: usize) {
        self.allocated.push((va, phys));
    }
    
    /// Commit the allocation - pages will NOT be freed on drop.
    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for MmapGuard {
    fn drop(&mut self) {
        if self.committed {
            return; // Success path - keep pages
        }
        
        // Failure path - clean up all allocated pages
        use los_hal::mmu::{PageAllocator, phys_to_virt, PageTable, tlb_flush_page};
        use crate::memory::FRAME_ALLOCATOR;
        
        for &(va, phys) in &self.allocated {
            // 1. Unmap the page (clear PTE)
            let l0_va = phys_to_virt(self.ttbr0);
            if let Ok(walk) = los_hal::mmu::walk_to_entry(
                unsafe { &mut *(l0_va as *mut PageTable) },
                va,
                3,
                false,
            ) {
                walk.table.entry_mut(walk.index).clear();
                tlb_flush_page(va);
            }
            
            // 2. Free the physical page
            FRAME_ALLOCATOR.free_page(phys);
        }
    }
}
```

---

## Acceptance Criteria

- [ ] `MmapGuard` struct compiles
- [ ] `new()` creates empty guard
- [ ] `track()` records VA/PA pairs
- [ ] `commit()` sets committed flag
- [ ] `Drop` does nothing if committed
- [ ] `Drop` unmaps and frees pages if not committed

---

## Testing

```bash
cargo build -p levitate-kernel
```

Integration tested via UoW 2.2.

---

## Next UoW

Proceed to **UoW 2.2** to integrate the guard into sys_mmap.
