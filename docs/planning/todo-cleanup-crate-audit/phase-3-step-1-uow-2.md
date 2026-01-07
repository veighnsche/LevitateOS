# UoW 1.2: Implement destroy_user_page_table

**Parent:** Phase 3, Step 1 (Page Table Teardown)  
**File:** `kernel/src/memory/user.rs`  
**Depends On:** UoW 1.1  
**Complexity:** Low

---

## Context

UoW 1.1 added `collect_page_table_entries` helper. Now we implement the actual `destroy_user_page_table` function that uses it.

---

## Task

Replace the stub `destroy_user_page_table` with a working implementation.

---

## Implementation

Replace the existing stub with:

```rust
/// TEAM_235: Free a user page table and all its mappings.
///
/// Walks the page table hierarchy, frees all mapped pages,
/// then frees the page tables themselves bottom-up.
///
/// # Safety
/// - `ttbr0_phys` must be a valid user L0 page table
/// - Must not be called while the page table is active (TTBR0)
pub unsafe fn destroy_user_page_table(ttbr0_phys: usize) -> Result<(), MmuError> {
    use los_hal::mmu::{PageAllocator, tlb_flush_all};
    
    let mut pages_to_free = alloc::vec::Vec::new();
    let mut tables_to_free = alloc::vec::Vec::new();
    
    // 1. Collect all entries starting from L0
    unsafe {
        collect_page_table_entries(
            ttbr0_phys,
            0, // Start at L0
            &mut pages_to_free,
            &mut tables_to_free,
        );
    }
    
    // 2. Free all leaf pages first
    for page_phys in pages_to_free {
        FRAME_ALLOCATOR.free_page(page_phys);
    }
    
    // 3. Free intermediate tables (already in bottom-up order from recursion)
    for table_phys in tables_to_free {
        FRAME_ALLOCATOR.free_page(table_phys);
    }
    
    // 4. Free the L0 table itself
    FRAME_ALLOCATOR.free_page(ttbr0_phys);
    
    // 5. Flush TLB to ensure no stale entries
    tlb_flush_all();
    
    Ok(())
}
```

---

## Acceptance Criteria

- [ ] Function compiles without errors
- [ ] All leaf pages freed via `FRAME_ALLOCATOR.free_page()`
- [ ] All intermediate tables freed
- [ ] L0 table freed last
- [ ] TLB flushed after teardown
- [ ] Returns `Ok(())`

---

## Testing

```bash
cargo build -p levitate-kernel
```

Full testing in UoW 1.3 (unit test).

---

## Next UoW

Proceed to **UoW 1.3** to add a unit test.
