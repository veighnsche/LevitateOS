# UoW 1.1: Add Recursive Page Table Walker Helper

**Parent:** Phase 3, Step 1 (Page Table Teardown)  
**File:** `kernel/src/memory/user.rs`  
**Complexity:** Low-Medium

---

## Context

The `destroy_user_page_table` function is currently a no-op stub that leaks pages. Before we can free pages, we need a way to walk the 4-level page table structure.

**Current stub location:**
```rust
// kernel/src/memory/user.rs:387-393
pub unsafe fn destroy_user_page_table(_ttbr0_phys: usize) -> Result<(), MmuError> {
    // TODO(TEAM_073): Implement full page table teardown
    // For now, we leak the pages - will be fixed when process cleanup is added
    Ok(())
}
```

---

## Task

Create a helper function that recursively walks page tables and collects information needed for cleanup.

---

## Implementation

Add this function **above** `destroy_user_page_table`:

```rust
/// TEAM_235: Recursively walk a page table and collect entries for cleanup.
///
/// # Arguments
/// * `table_phys` - Physical address of the page table
/// * `level` - Current level (0=L0, 1=L1, 2=L2, 3=L3)
/// * `pages_to_free` - Accumulator for leaf page physical addresses
/// * `tables_to_free` - Accumulator for table physical addresses (freed last)
///
/// # Safety
/// - `table_phys` must be a valid page table at the given level
unsafe fn collect_page_table_entries(
    table_phys: usize,
    level: usize,
    pages_to_free: &mut alloc::vec::Vec<usize>,
    tables_to_free: &mut alloc::vec::Vec<usize>,
) {
    use los_hal::mmu::{PageTable, phys_to_virt, ENTRIES_PER_TABLE};
    
    let table_va = phys_to_virt(table_phys);
    let table = unsafe { &*(table_va as *const PageTable) };
    
    for i in 0..ENTRIES_PER_TABLE {
        let entry = table.entry(i);
        
        if !entry.is_valid() {
            continue;
        }
        
        let entry_phys = entry.address();
        
        if level == 3 {
            // L3: These are leaf pages - add to free list
            pages_to_free.push(entry_phys);
        } else if entry.is_table() {
            // Intermediate table descriptor - recurse
            unsafe {
                collect_page_table_entries(
                    entry_phys,
                    level + 1,
                    pages_to_free,
                    tables_to_free,
                );
            }
            // Add child table to free list (will be freed after its contents)
            tables_to_free.push(entry_phys);
        } else {
            // Block mapping (L1 1GB or L2 2MB) - add to free list
            // Note: User space typically doesn't use blocks, but handle anyway
            pages_to_free.push(entry_phys);
        }
    }
}
```

---

## Acceptance Criteria

- [ ] Function compiles without errors
- [ ] Function handles all 4 levels (L0-L3)
- [ ] L3 entries added to `pages_to_free`
- [ ] Intermediate tables added to `tables_to_free` (after recursing)
- [ ] Block entries (if any) added to `pages_to_free`
- [ ] Invalid entries skipped

---

## Testing

This UoW adds a helper function. Testing will be done in UoW 1.5 after the full implementation is complete.

**Compile check:**
```bash
cargo build -p levitate-kernel
```

---

## Next UoW

After this UoW, proceed to **UoW 1.2** to implement the actual freeing of collected pages.
